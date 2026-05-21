use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::barrier_profile_registry::{
    BarrierProfileRegistry, load_barrier_profile_registry_from_path_or_config,
};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceRow,
    ComparableEvidenceSourceClass,
};
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceItem, MultiRowOfficialEvidenceSet,
    load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryItem, OfficialReadyRowInventoryReport,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialDiversitySweepConfig {
    pub sweep_id: String,
    #[serde(default)]
    pub diversity_gap_config_path: Option<String>,
    #[serde(default)]
    pub diversity_gap_map_path: Option<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default)]
    pub multi_row_set_config_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_pack_paths: Vec<String>,
    #[serde(default)]
    pub canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub provenance_paths: Vec<String>,
    #[serde(default)]
    pub preflight_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_new_rows")]
    pub max_new_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_timeframes")]
    pub max_timeframes: usize,
    #[serde(default = "default_max_horizons")]
    pub max_horizons: usize,
    #[serde(default = "default_max_jobs")]
    pub max_jobs: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub prefer_existing_official_rows: bool,
    #[serde(default = "default_true")]
    pub prefer_local_canonical_csv: bool,
    #[serde(default = "default_true")]
    pub allow_provider_job_generation: bool,
    #[serde(default)]
    pub run_provider_collection_jobs: bool,
    #[serde(default = "default_true")]
    pub run_local_extension_jobs: bool,
    #[serde(default)]
    pub allow_diagnostic_profiles: bool,
    #[serde(default)]
    pub allow_exploratory_profiles: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiversitySelectionReason {
    AddsNewSymbol,
    AddsNewTimeframe,
    AddsNewHorizon,
    AddsStopLossCandidate,
    AddsTimeExpiredCandidate,
    ReducesOutcomeConcentration,
    ReducesSymbolConcentration,
    ImprovesCounterfactualDepth,
    ExistingDataAvailable,
    FutureWindowAvailable,
    SourceIneligible,
    DiagnosticOnly,
    BudgetExceeded,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialDiversityCandidateRow {
    pub candidate_id: String,
    pub symbol: String,
    pub market: crate::data::ProviderMarket,
    #[serde(default)]
    pub venue: Option<String>,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub timestamp_ms: u64,
    pub source_class: ComparableEvidenceSourceClass,
    pub available_candle_window: bool,
    #[serde(default)]
    pub preregistered_profile_id: Option<String>,
    pub expected_official_complete_possible: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub selection_reasons: Vec<DiversitySelectionReason>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialDiversityRowSelectorConfig {
    pub selector_id: String,
    #[serde(default)]
    pub sweep_config_path: Option<String>,
    #[serde(default)]
    pub candidate_sources: Vec<String>,
    #[serde(default = "default_max_candidates")]
    pub max_candidates: usize,
    #[serde(default = "default_max_per_symbol")]
    pub max_per_symbol: usize,
    #[serde(default = "default_max_per_timeframe")]
    pub max_per_timeframe: usize,
    #[serde(default = "default_max_per_outcome_label_hint")]
    pub max_per_outcome_label_hint: usize,
    #[serde(default = "default_true")]
    pub require_preregistered_profile: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialDiversityRowSelectorStatus {
    CandidatesSelected,
    NeedMoreCandidateRows,
    NeedMoreCandleData,
    BudgetBlocked,
    SourceIneligible,
    DiagnosticOnly,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialDiversityRowSelectorReport {
    pub selector_id: String,
    pub selected_candidates: Vec<OfficialDiversityCandidateRow>,
    pub skipped_candidates: Vec<OfficialDiversityCandidateRow>,
    pub added_symbol_potential: usize,
    pub added_timeframe_potential: usize,
    pub added_horizon_potential: usize,
    pub expected_rows: usize,
    pub expected_official_complete_rows: usize,
    pub selector_status: OfficialDiversityRowSelectorStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialDiversityRowSelector;

impl Default for OfficialDiversitySweepConfig {
    fn default() -> Self {
        Self {
            sweep_id: "official-diversity-sweep-plan".to_string(),
            diversity_gap_config_path: None,
            diversity_gap_map_path: None,
            barrier_profile_registry_path: None,
            multi_row_set_config_paths: Vec::new(),
            official_candle_pack_paths: Vec::new(),
            canonical_csv_paths: Vec::new(),
            provenance_paths: Vec::new(),
            preflight_report_paths: Vec::new(),
            output_root: default_output_root(),
            max_new_rows: default_max_new_rows(),
            max_symbols: default_max_symbols(),
            max_timeframes: default_max_timeframes(),
            max_horizons: default_max_horizons(),
            max_jobs: default_max_jobs(),
            max_bytes: default_max_bytes(),
            prefer_existing_official_rows: true,
            prefer_local_canonical_csv: true,
            allow_provider_job_generation: true,
            run_provider_collection_jobs: false,
            run_local_extension_jobs: true,
            allow_diagnostic_profiles: false,
            allow_exploratory_profiles: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialDiversitySweepConfig {
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
        if self.sweep_id.trim().is_empty() {
            return Err("official diversity sweep id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| is_remote_path(path))
        {
            return Err("official diversity sweep config paths must be local".to_string());
        }
        if self.max_new_rows == 0 || self.max_new_rows > 1000 {
            return Err(
                "official diversity sweep max_new_rows must be between 1 and 1000".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > 10 {
            return Err(
                "official diversity sweep max_symbols must be between 1 and 10".to_string(),
            );
        }
        if self.max_timeframes == 0 || self.max_timeframes > 5 {
            return Err(
                "official diversity sweep max_timeframes must be between 1 and 5".to_string(),
            );
        }
        if self.max_horizons == 0 || self.max_horizons > 5 {
            return Err(
                "official diversity sweep max_horizons must be between 1 and 5".to_string(),
            );
        }
        if self.max_jobs == 0 || self.max_jobs > 1000 {
            return Err("official diversity sweep max_jobs must be between 1 and 1000".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official diversity sweep max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.sweep_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.diversity_gap_config_path
            .iter()
            .chain(self.diversity_gap_map_path.iter())
            .chain(self.barrier_profile_registry_path.iter())
            .chain(self.multi_row_set_config_paths.iter())
            .chain(self.official_candle_pack_paths.iter())
            .chain(self.canonical_csv_paths.iter())
            .chain(self.provenance_paths.iter())
            .chain(self.preflight_report_paths.iter())
            .cloned()
            .collect()
    }
}

impl Default for OfficialDiversityRowSelectorConfig {
    fn default() -> Self {
        Self {
            selector_id: "official-diversity-row-selector".to_string(),
            sweep_config_path: None,
            candidate_sources: Vec::new(),
            max_candidates: default_max_candidates(),
            max_per_symbol: default_max_per_symbol(),
            max_per_timeframe: default_max_per_timeframe(),
            max_per_outcome_label_hint: default_max_per_outcome_label_hint(),
            require_preregistered_profile: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialDiversityRowSelectorConfig {
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
        if self.selector_id.trim().is_empty() {
            return Err("official diversity row selector id must not be empty".to_string());
        }
        let output_root = "target".to_string();
        let all_paths = self
            .candidate_sources
            .iter()
            .chain(self.sweep_config_path.iter())
            .chain(std::iter::once(&output_root))
            .collect::<Vec<_>>();
        if all_paths.iter().any(|path| is_remote_path(path)) {
            return Err("official diversity row selector paths must be local".to_string());
        }
        if self.max_candidates == 0 || self.max_candidates > 1000 {
            return Err(
                "official diversity row selector max_candidates must be between 1 and 1000"
                    .to_string(),
            );
        }
        if self.max_per_symbol == 0 || self.max_per_symbol > 100 {
            return Err(
                "official diversity row selector max_per_symbol must be between 1 and 100"
                    .to_string(),
            );
        }
        if self.max_per_timeframe == 0 || self.max_per_timeframe > 100 {
            return Err(
                "official diversity row selector max_per_timeframe must be between 1 and 100"
                    .to_string(),
            );
        }
        Ok(())
    }
}

impl OfficialDiversityRowSelector {
    pub fn run(
        &self,
        config: &OfficialDiversityRowSelectorConfig,
    ) -> Result<OfficialDiversityRowSelectorReport, String> {
        config.validate()?;
        let sweep_config = config
            .sweep_config_path
            .as_deref()
            .map(|path| OfficialDiversitySweepConfig::from_toml_path(Path::new(path)))
            .transpose()?;
        let registry = sweep_config
            .as_ref()
            .and_then(|sweep| sweep.barrier_profile_registry_path.as_deref())
            .map(load_barrier_profile_registry_from_path_or_config)
            .transpose()?;
        let baseline_set = sweep_config
            .as_ref()
            .and_then(|sweep| sweep.multi_row_set_config_paths.first())
            .map(|path| load_multi_row_official_evidence_set_from_path_or_config(path))
            .transpose()?;
        let candidate_sources = if config.candidate_sources.is_empty() {
            sweep_config
                .as_ref()
                .map(|sweep| sweep.multi_row_set_config_paths.clone())
                .unwrap_or_default()
        } else {
            config.candidate_sources.clone()
        };
        let candidates = load_candidates(&candidate_sources, registry.as_ref())?;
        Ok(self.run_from_candidates(config, baseline_set.as_ref(), registry.as_ref(), candidates))
    }

    pub fn run_from_candidates(
        &self,
        config: &OfficialDiversityRowSelectorConfig,
        baseline_set: Option<&MultiRowOfficialEvidenceSet>,
        registry: Option<&BarrierProfileRegistry>,
        mut candidates: Vec<OfficialDiversityCandidateRow>,
    ) -> OfficialDiversityRowSelectorReport {
        let baseline_symbols = baseline_set
            .map(|set| {
                set.items
                    .iter()
                    .filter(|item| item.official_complete)
                    .map(|item| item.symbol.clone())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let baseline_timeframes = baseline_set
            .map(|set| {
                set.items
                    .iter()
                    .filter(|item| item.official_complete)
                    .map(|item| item.timeframe.clone())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let baseline_horizons = baseline_set
            .map(|set| {
                set.items
                    .iter()
                    .filter(|item| item.official_complete)
                    .map(|item| item.horizon_bars)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let default_profile_id = registry
            .and_then(|registry| registry.official_profile(None))
            .map(|profile| profile.profile_id.clone());

        for candidate in &mut candidates {
            if candidate.preregistered_profile_id.is_none() {
                candidate.preregistered_profile_id = default_profile_id.clone();
            }
            if !candidate.available_candle_window {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::NeedMoreEvidence);
            }
        }

        candidates.sort_by(|left, right| {
            candidate_priority(
                left,
                &baseline_symbols,
                &baseline_timeframes,
                &baseline_horizons,
            )
            .cmp(&candidate_priority(
                right,
                &baseline_symbols,
                &baseline_timeframes,
                &baseline_horizons,
            ))
            .reverse()
            .then(left.candidate_id.cmp(&right.candidate_id))
        });

        let mut selected_candidates = Vec::new();
        let mut skipped_candidates = Vec::new();
        let mut selected_symbols = baseline_symbols.clone();
        let mut selected_timeframes = baseline_timeframes.clone();
        let mut selected_horizons = baseline_horizons.clone();
        let mut per_symbol = BTreeMap::<String, usize>::new();
        let mut per_timeframe = BTreeMap::<String, usize>::new();

        for mut candidate in candidates {
            let source_ineligible = candidate.source_class
                != ComparableEvidenceSourceClass::OfficialNonCrypto
                || !candidate.expected_official_complete_possible;
            if source_ineligible {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::SourceIneligible);
                skipped_candidates.push(candidate);
                continue;
            }
            if candidate.diagnostic_only {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::DiagnosticOnly);
                skipped_candidates.push(candidate);
                continue;
            }
            if config.require_preregistered_profile && candidate.preregistered_profile_id.is_none()
            {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::SourceIneligible);
                skipped_candidates.push(candidate);
                continue;
            }
            if selected_candidates.len() >= config.max_candidates
                || per_symbol
                    .get(&candidate.symbol)
                    .copied()
                    .unwrap_or_default()
                    >= config.max_per_symbol
                || per_timeframe
                    .get(&candidate.timeframe)
                    .copied()
                    .unwrap_or_default()
                    >= config.max_per_timeframe
            {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::BudgetExceeded);
                skipped_candidates.push(candidate);
                continue;
            }
            if !selected_symbols.contains(&candidate.symbol) {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::AddsNewSymbol);
            }
            if !selected_timeframes.contains(&candidate.timeframe) {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::AddsNewTimeframe);
            }
            if !selected_horizons.contains(&candidate.horizon_bars) {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::AddsNewHorizon);
            }
            if candidate.available_candle_window {
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::ExistingDataAvailable);
                candidate
                    .selection_reasons
                    .push(DiversitySelectionReason::FutureWindowAvailable);
            }
            selected_symbols.insert(candidate.symbol.clone());
            selected_timeframes.insert(candidate.timeframe.clone());
            selected_horizons.insert(candidate.horizon_bars);
            *per_symbol.entry(candidate.symbol.clone()).or_insert(0usize) += 1;
            *per_timeframe
                .entry(candidate.timeframe.clone())
                .or_insert(0usize) += 1;
            selected_candidates.push(candidate);
        }

        selected_candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        skipped_candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));

        let added_symbol_potential = selected_symbols
            .len()
            .saturating_sub(baseline_symbols.len());
        let added_timeframe_potential = selected_timeframes
            .len()
            .saturating_sub(baseline_timeframes.len());
        let added_horizon_potential = selected_horizons
            .len()
            .saturating_sub(baseline_horizons.len());
        let expected_rows = selected_candidates.len();
        let expected_official_complete_rows = selected_candidates
            .iter()
            .filter(|candidate| candidate.expected_official_complete_possible)
            .count();
        let selector_status = determine_selector_status(&selected_candidates, &skipped_candidates);
        let warnings = vec![
            "official diversity row selection is research-only and must not peek at future outcomes for official selection"
                .to_string(),
        ];

        OfficialDiversityRowSelectorReport {
            selector_id: config.selector_id.clone(),
            selected_candidates,
            skipped_candidates,
            added_symbol_potential,
            added_timeframe_potential,
            added_horizon_potential,
            expected_rows,
            expected_official_complete_rows,
            selector_status,
            warnings,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::DeterministicPath,
                        ReasonCode::LocalFileOnly,
                        ReasonCode::OfficialEvidenceCounted,
                    ])
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl OfficialDiversityRowSelectorReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.selector_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("selector_id={}", self.selector_id),
            format!("selected_candidates={}", self.selected_candidates.len()),
            format!("skipped_candidates={}", self.skipped_candidates.len()),
            format!("added_symbol_potential={}", self.added_symbol_potential),
            format!(
                "added_timeframe_potential={}",
                self.added_timeframe_potential
            ),
            format!("added_horizon_potential={}", self.added_horizon_potential),
            format!("expected_rows={}", self.expected_rows),
            format!(
                "expected_official_complete_rows={}",
                self.expected_official_complete_rows
            ),
            format!("selector_status={:?}", self.selector_status),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.selected_candidates.iter().map(candidate_to_line));
        lines.extend(
            self.skipped_candidates
                .iter()
                .map(|candidate| format!("skipped:{}", candidate_to_line(candidate))),
        );
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_diversity_row_selector.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_diversity_row_selector.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_official_diversity_row_selector_report_from_path_or_config(
    path: &str,
) -> Result<OfficialDiversityRowSelectorReport, String> {
    if path.ends_with(".json") {
        OfficialDiversityRowSelectorReport::from_json_path(Path::new(path))
    } else {
        OfficialDiversityRowSelectorConfig::from_toml_path(Path::new(path))
            .and_then(|config| OfficialDiversityRowSelector::default().run(&config))
    }
}

fn candidate_priority(
    candidate: &OfficialDiversityCandidateRow,
    baseline_symbols: &BTreeSet<String>,
    baseline_timeframes: &BTreeSet<String>,
    baseline_horizons: &BTreeSet<usize>,
) -> (usize, usize, usize, usize, bool) {
    (
        usize::from(!baseline_symbols.contains(&candidate.symbol)),
        usize::from(!baseline_timeframes.contains(&candidate.timeframe)),
        usize::from(!baseline_horizons.contains(&candidate.horizon_bars)),
        usize::from(candidate.available_candle_window),
        !candidate.diagnostic_only,
    )
}

fn determine_selector_status(
    selected: &[OfficialDiversityCandidateRow],
    skipped: &[OfficialDiversityCandidateRow],
) -> OfficialDiversityRowSelectorStatus {
    if !selected.is_empty() {
        return OfficialDiversityRowSelectorStatus::CandidatesSelected;
    }
    if skipped.iter().any(|candidate| {
        candidate
            .selection_reasons
            .contains(&DiversitySelectionReason::BudgetExceeded)
    }) {
        return OfficialDiversityRowSelectorStatus::BudgetBlocked;
    }
    if skipped.iter().all(|candidate| candidate.diagnostic_only) && !skipped.is_empty() {
        return OfficialDiversityRowSelectorStatus::DiagnosticOnly;
    }
    if skipped.iter().any(|candidate| {
        candidate
            .selection_reasons
            .contains(&DiversitySelectionReason::SourceIneligible)
    }) {
        return OfficialDiversityRowSelectorStatus::SourceIneligible;
    }
    if skipped
        .iter()
        .any(|candidate| !candidate.available_candle_window)
    {
        return OfficialDiversityRowSelectorStatus::NeedMoreCandleData;
    }
    if skipped.is_empty() {
        OfficialDiversityRowSelectorStatus::NeedMoreCandidateRows
    } else {
        OfficialDiversityRowSelectorStatus::NeedMoreEvidence
    }
}

fn candidate_to_line(candidate: &OfficialDiversityCandidateRow) -> String {
    format!(
        "candidate_id={};symbol={};timeframe={};horizon_bars={};available_candle_window={};source_class={:?};diagnostic_only={};preregistered_profile_id={};selection_reasons={}",
        candidate.candidate_id,
        candidate.symbol,
        candidate.timeframe,
        candidate.horizon_bars,
        candidate.available_candle_window,
        candidate.source_class,
        candidate.diagnostic_only,
        candidate
            .preregistered_profile_id
            .clone()
            .unwrap_or_default(),
        candidate
            .selection_reasons
            .iter()
            .map(|reason| format!("{reason:?}"))
            .collect::<Vec<_>>()
            .join("|"),
    )
}

fn load_candidates(
    paths: &[String],
    registry: Option<&BarrierProfileRegistry>,
) -> Result<Vec<OfficialDiversityCandidateRow>, String> {
    let mut candidates = Vec::new();
    for path in paths {
        let mut loaded = load_candidates_from_path(path, registry)?;
        candidates.append(&mut loaded);
    }
    candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    candidates.dedup_by(|left, right| left.candidate_id == right.candidate_id);
    Ok(candidates)
}

fn load_candidates_from_path(
    path: &str,
    registry: Option<&BarrierProfileRegistry>,
) -> Result<Vec<OfficialDiversityCandidateRow>, String> {
    let default_profile_id = registry
        .and_then(|registry| registry.official_profile(None))
        .map(|profile| profile.profile_id.clone());
    if path.ends_with(".json") {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if let Ok(set) = serde_json::from_str::<MultiRowOfficialEvidenceSet>(&text) {
            return Ok(set
                .items
                .iter()
                .map(|item| candidate_from_multi_row_item(item, default_profile_id.clone()))
                .collect());
        }
        if let Ok(report) = serde_json::from_str::<OfficialReadyRowInventoryReport>(&text) {
            return Ok(report
                .items
                .iter()
                .map(|item| candidate_from_inventory_item(item, default_profile_id.clone()))
                .collect());
        }
        if let Ok(bundle) = serde_json::from_str::<ComparableCommitteeEvidenceBundle>(&text) {
            return Ok(bundle
                .rows
                .iter()
                .map(|row| candidate_from_comparable_row(row, default_profile_id.clone()))
                .collect());
        }
        return Err(format!(
            "official diversity row selector could not parse candidate source '{}'",
            path
        ));
    }
    let set = load_multi_row_official_evidence_set_from_path_or_config(path)?;
    Ok(set
        .items
        .iter()
        .map(|item| candidate_from_multi_row_item(item, default_profile_id.clone()))
        .collect())
}

fn candidate_from_multi_row_item(
    item: &MultiRowOfficialEvidenceItem,
    preregistered_profile_id: Option<String>,
) -> OfficialDiversityCandidateRow {
    let diagnostic_only = item.diagnostic_only
        || item.source_class != ComparableEvidenceSourceClass::OfficialNonCrypto;
    OfficialDiversityCandidateRow {
        candidate_id: item.row_id.clone(),
        symbol: item.symbol.clone(),
        market: item.market,
        venue: item.venue.clone(),
        timeframe: item.timeframe.clone(),
        horizon_bars: item.horizon_bars,
        timestamp_ms: item.timestamp_ms,
        source_class: item.source_class,
        available_candle_window: item.future_window_sufficient && item.no_lookahead_safe,
        preregistered_profile_id,
        expected_official_complete_possible: item.no_lookahead_safe
            && item.future_window_sufficient
            && item.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto,
        diagnostic_only,
        selection_reasons: Vec::new(),
        reason_codes: stable_reason_codes(&item.reason_codes),
    }
}

fn candidate_from_inventory_item(
    item: &OfficialReadyRowInventoryItem,
    preregistered_profile_id: Option<String>,
) -> OfficialDiversityCandidateRow {
    let diagnostic_only = item.source_class != ComparableEvidenceSourceClass::OfficialNonCrypto
        || item.summary_derived;
    OfficialDiversityCandidateRow {
        candidate_id: item.row_id.clone(),
        symbol: item.symbol.clone(),
        market: item.market,
        venue: item.venue.clone(),
        timeframe: item.timeframe.clone(),
        horizon_bars: item.horizon_bars,
        timestamp_ms: item.timestamp_ms,
        source_class: item.source_class,
        available_candle_window: item.official_ready_match && item.no_lookahead_safe,
        preregistered_profile_id,
        expected_official_complete_possible: item.official_ready_match
            && item.no_lookahead_safe
            && item.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto,
        diagnostic_only,
        selection_reasons: Vec::new(),
        reason_codes: stable_reason_codes(&item.reason_codes),
    }
}

fn candidate_from_comparable_row(
    row: &ComparableCommitteeEvidenceRow,
    preregistered_profile_id: Option<String>,
) -> OfficialDiversityCandidateRow {
    let diagnostic_only =
        row.diagnostic_only || row.source_class != ComparableEvidenceSourceClass::OfficialNonCrypto;
    OfficialDiversityCandidateRow {
        candidate_id: row.row_id.clone(),
        symbol: row.symbol.clone(),
        market: row.market,
        venue: None,
        timeframe: row.timeframe.clone(),
        horizon_bars: row.horizon_bars,
        timestamp_ms: row.timestamp_ms,
        source_class: row.source_class,
        available_candle_window: row.candle_coverage_available && row.no_lookahead_safe,
        preregistered_profile_id,
        expected_official_complete_possible: row.official_readiness_eligible
            && row.candle_coverage_available
            && row.no_lookahead_safe,
        diagnostic_only,
        selection_reasons: Vec::new(),
        reason_codes: stable_reason_codes(&row.reason_codes),
    }
}

fn default_output_root() -> String {
    "target/soma_official_diversity_row_selector".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_new_rows() -> usize {
    100
}

fn default_max_symbols() -> usize {
    10
}

fn default_max_timeframes() -> usize {
    5
}

fn default_max_horizons() -> usize {
    5
}

fn default_max_jobs() -> usize {
    100
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_max_candidates() -> usize {
    100
}

fn default_max_per_symbol() -> usize {
    5
}

fn default_max_per_timeframe() -> usize {
    10
}

fn default_max_per_outcome_label_hint() -> usize {
    10
}

fn is_remote_path(value: &str) -> bool {
    value.contains("://")
}
