use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::{
    EvidenceSourceKind, PreflightFinalStatus, PreflightReport, ProviderKind, ProviderMarket,
};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::comparable_evidence_builder::ComparableEvidenceBuilder;
use super::complete_row_closure_bundle::CompleteRowClosureBundle;
use super::official_candle_coverage_pack::{
    OfficialCandleSeriesDescriptor, OfficialCandleSeriesSourceClass,
    load_candle_csv_timestamp_series, load_pack_from_path_or_config, normalize_symbol,
    normalize_timeframe_label, timeframe_seconds,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryConfig, OfficialReadyRowInventoryItem,
    OfficialReadyRowInventoryReport, OfficialReadyRowInventoryRunner,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FutureWindowRequirementConfig {
    pub requirement_id: String,
    #[serde(default)]
    pub official_ready_inventory_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_closure_paths: Vec<String>,
    #[serde(default)]
    pub scenario_materialization_v3_paths: Vec<String>,
    #[serde(default)]
    pub candle_coverage_pack_paths: Vec<String>,
    #[serde(default)]
    pub official_ready_match_closure_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_horizon_bars")]
    pub default_horizon_bars: usize,
    #[serde(default = "default_take_profit_pct")]
    pub default_take_profit_pct: f64,
    #[serde(default = "default_stop_loss_pct")]
    pub default_stop_loss_pct: f64,
    #[serde(default = "default_cost_bps")]
    pub default_cost_bps: f64,
    #[serde(default = "default_slippage_bps")]
    pub default_slippage_bps: f64,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_official_ready_match: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub allow_controlled_diagnostic: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FutureWindowGapKind {
    SufficientFutureBars,
    MissingFutureBars,
    MissingCandleWindow,
    MissingScenarioTimestamp,
    MissingHorizon,
    HorizonMismatch,
    TimestampOutsideRange,
    TimeframeMismatch,
    SymbolMismatch,
    NoLookaheadViolation,
    SourceIneligible,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FutureWindowRequirementItem {
    pub row_id: String,
    #[serde(default)]
    pub scenario_row_id: Option<String>,
    #[serde(default)]
    pub comparable_row_id: Option<String>,
    #[serde(default)]
    pub candle_series_id: Option<String>,
    pub symbol: String,
    pub market: ProviderMarket,
    #[serde(default)]
    pub venue: Option<String>,
    pub timeframe: String,
    pub timestamp_ms: u64,
    pub horizon_bars: usize,
    pub current_available_future_bars: usize,
    pub required_future_bars: usize,
    pub missing_future_bars: usize,
    pub required_start_timestamp_ms: u64,
    pub required_end_timestamp_ms: u64,
    pub can_extend_from_existing_csv: bool,
    pub can_extend_from_provider_collection: bool,
    pub gap_kind: FutureWindowGapKind,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FutureWindowRequirementStatus {
    HealthyFutureWindows,
    NeedLongerFutureWindow,
    NeedOfficialCandleExtension,
    NeedProviderCollection,
    NeedTimestampAlignment,
    NeedHorizonAlignment,
    SourceIneligible,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FutureWindowRequirementReport {
    pub requirement_id: String,
    pub items: Vec<FutureWindowRequirementItem>,
    pub total_items: usize,
    pub rows_with_sufficient_future_window: usize,
    pub rows_missing_future_window: usize,
    pub rows_extendable_from_local_csv: usize,
    pub rows_extendable_from_provider: usize,
    pub rows_source_ineligible: usize,
    pub no_lookahead_blocked_rows: usize,
    pub requirement_status: FutureWindowRequirementStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FutureWindowRequirementRunner;

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedFutureWindowInputs {
    pub inventory: OfficialReadyRowInventoryReport,
    pub descriptors: BTreeMap<String, OfficialCandleSeriesDescriptor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FutureBar {
    pub timestamp_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Default for FutureWindowRequirementConfig {
    fn default() -> Self {
        Self {
            requirement_id: "future-window-requirements".to_string(),
            official_ready_inventory_paths: Vec::new(),
            complete_row_closure_paths: Vec::new(),
            scenario_materialization_v3_paths: Vec::new(),
            candle_coverage_pack_paths: Vec::new(),
            official_ready_match_closure_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            output_root: default_output_root(),
            default_horizon_bars: default_horizon_bars(),
            default_take_profit_pct: default_take_profit_pct(),
            default_stop_loss_pct: default_stop_loss_pct(),
            default_cost_bps: default_cost_bps(),
            default_slippage_bps: default_slippage_bps(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_official_ready_match: true,
            require_no_lookahead_safe: true,
            allow_controlled_diagnostic: false,
            allow_crypto_only: true,
            allow_yfinance_research: false,
            allow_fixture: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl FutureWindowRequirementConfig {
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
        if self.requirement_id.trim().is_empty() {
            return Err("future window requirement id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("future window requirement paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err("future window requirement max_rows must be between 1 and 500".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "future window requirement max_symbols must be between 1 and 5".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "future window requirement max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.requirement_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.official_ready_inventory_paths
            .iter()
            .chain(self.complete_row_closure_paths.iter())
            .chain(self.scenario_materialization_v3_paths.iter())
            .chain(self.candle_coverage_pack_paths.iter())
            .chain(self.official_ready_match_closure_paths.iter())
            .chain(self.comparable_evidence_bundle_paths.iter())
            .cloned()
            .collect()
    }
}

impl FutureWindowRequirementRunner {
    pub fn run(
        &self,
        config: &FutureWindowRequirementConfig,
    ) -> Result<FutureWindowRequirementReport, String> {
        config.validate()?;
        let loaded = load_future_window_inputs(config)?;
        self.run_from_inventory(config, &loaded.inventory, &loaded.descriptors)
    }

    pub fn run_from_inventory(
        &self,
        config: &FutureWindowRequirementConfig,
        inventory: &OfficialReadyRowInventoryReport,
        descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
    ) -> Result<FutureWindowRequirementReport, String> {
        config.validate()?;
        if inventory.items.len() > config.max_rows {
            return Err(format!(
                "future window requirement loaded {} rows which exceeds max_rows {}",
                inventory.items.len(),
                config.max_rows
            ));
        }
        let unique_symbols = inventory
            .items
            .iter()
            .map(|item| item.symbol.clone())
            .collect::<BTreeSet<_>>();
        if unique_symbols.len() > config.max_symbols {
            return Err(format!(
                "future window requirement loaded {} symbols which exceeds max_symbols {}",
                unique_symbols.len(),
                config.max_symbols
            ));
        }
        let storage_bytes = input_storage_bytes(&config.all_paths());
        if storage_bytes > config.max_bytes {
            return Err(format!(
                "future window requirement input size {} exceeds max_bytes {}",
                storage_bytes, config.max_bytes
            ));
        }

        let mut items = inventory
            .items
            .iter()
            .filter(|item| {
                if config.require_official_ready_match {
                    item.official_ready_match
                } else {
                    item.official_ready_match || item.benchmark_ready_match
                }
            })
            .map(|item| build_requirement_item(config, item, descriptors))
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.row_id
                .cmp(&right.row_id)
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timestamp_ms.cmp(&right.timestamp_ms))
                .then(left.timeframe.cmp(&right.timeframe))
        });

        let total_items = items.len();
        let rows_with_sufficient_future_window = items
            .iter()
            .filter(|item| item.gap_kind == FutureWindowGapKind::SufficientFutureBars)
            .count();
        let rows_missing_future_window = items
            .iter()
            .filter(|item| item.gap_kind == FutureWindowGapKind::MissingFutureBars)
            .count();
        let rows_extendable_from_local_csv = items
            .iter()
            .filter(|item| item.missing_future_bars > 0 && item.can_extend_from_existing_csv)
            .count();
        let rows_extendable_from_provider = items
            .iter()
            .filter(|item| item.missing_future_bars > 0 && item.can_extend_from_provider_collection)
            .count();
        let rows_source_ineligible = items
            .iter()
            .filter(|item| item.gap_kind == FutureWindowGapKind::SourceIneligible)
            .count();
        let no_lookahead_blocked_rows = items
            .iter()
            .filter(|item| item.gap_kind == FutureWindowGapKind::NoLookaheadViolation)
            .count();
        let requirement_status = determine_status(&items);

        Ok(FutureWindowRequirementReport {
            requirement_id: config.requirement_id.clone(),
            items,
            total_items,
            rows_with_sufficient_future_window,
            rows_missing_future_window,
            rows_extendable_from_local_csv,
            rows_extendable_from_provider,
            rows_source_ineligible,
            no_lookahead_blocked_rows,
            requirement_status,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl FutureWindowRequirementReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.requirement_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("requirement_id={}", self.requirement_id),
            format!("total_items={}", self.total_items),
            format!(
                "rows_with_sufficient_future_window={}",
                self.rows_with_sufficient_future_window
            ),
            format!(
                "rows_missing_future_window={}",
                self.rows_missing_future_window
            ),
            format!(
                "rows_extendable_from_local_csv={}",
                self.rows_extendable_from_local_csv
            ),
            format!(
                "rows_extendable_from_provider={}",
                self.rows_extendable_from_provider
            ),
            format!("rows_source_ineligible={}", self.rows_source_ineligible),
            format!(
                "no_lookahead_blocked_rows={}",
                self.no_lookahead_blocked_rows
            ),
            format!("requirement_status={:?}", self.requirement_status),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "row_id={};symbol={};timeframe={};timestamp_ms={};horizon_bars={};current_available_future_bars={};required_future_bars={};missing_future_bars={};required_start_timestamp_ms={};required_end_timestamp_ms={};can_extend_from_existing_csv={};can_extend_from_provider_collection={};gap_kind={:?}",
                item.row_id,
                item.symbol,
                item.timeframe,
                item.timestamp_ms,
                item.horizon_bars,
                item.current_available_future_bars,
                item.required_future_bars,
                item.missing_future_bars,
                item.required_start_timestamp_ms,
                item.required_end_timestamp_ms,
                item.can_extend_from_existing_csv,
                item.can_extend_from_provider_collection,
                item.gap_kind,
            )
        }));
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
            output_dir.join("future_window_requirements.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("future_window_requirement_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_future_window_requirement_from_path_or_config(
    path: &str,
) -> Result<FutureWindowRequirementReport, String> {
    if path.ends_with(".json") {
        FutureWindowRequirementReport::from_json_path(Path::new(path))
    } else {
        FutureWindowRequirementConfig::from_toml_path(Path::new(path))
            .and_then(|config| FutureWindowRequirementRunner::default().run(&config))
    }
}

pub fn load_future_window_inputs(
    config: &FutureWindowRequirementConfig,
) -> Result<LoadedFutureWindowInputs, String> {
    config.validate()?;
    let mut inventory_reports = Vec::new();
    for path in &config.official_ready_inventory_paths {
        inventory_reports.push(load_inventory_path(path)?);
    }
    for path in &config.complete_row_closure_paths {
        if path.ends_with(".json") {
            let bundle = CompleteRowClosureBundle::from_json_path(Path::new(path))?;
            inventory_reports.push(bundle.inventory_report);
        }
    }

    let descriptors = load_descriptor_map_from_paths(&config.candle_coverage_pack_paths)?;

    let inventory = if let Some(first) = inventory_reports.into_iter().next() {
        first
    } else {
        let rows = load_rows_from_paths(&config.comparable_evidence_bundle_paths)?;
        let inventory_config = OfficialReadyRowInventoryConfig {
            inventory_id: format!("{}-inventory", config.requirement_id),
            output_root: config.output_root.clone(),
            require_official_ready_match: config.require_official_ready_match,
            require_no_lookahead_safe: config.require_no_lookahead_safe,
            allow_controlled_diagnostic: config.allow_controlled_diagnostic,
            allow_crypto_only: config.allow_crypto_only,
            allow_yfinance_research: config.allow_yfinance_research,
            allow_fixture: config.allow_fixture,
            max_rows: config.max_rows,
            max_symbols: config.max_symbols,
            max_bytes: config.max_bytes,
            reason_codes: config.reason_codes.clone(),
            ..OfficialReadyRowInventoryConfig::default()
        };
        OfficialReadyRowInventoryRunner::default().run_from_rows(
            &inventory_config,
            &rows,
            &BTreeMap::new(),
            &descriptors,
        )?
    };

    Ok(LoadedFutureWindowInputs {
        inventory,
        descriptors,
    })
}

pub fn load_descriptor_map_from_paths(
    paths: &[String],
) -> Result<BTreeMap<String, OfficialCandleSeriesDescriptor>, String> {
    let mut descriptors = BTreeMap::new();
    for path in paths {
        if path.ends_with(".csv") {
            let descriptor = descriptor_from_csv_path(path)?;
            descriptors.insert(descriptor.candle_series_id.clone(), descriptor);
        } else {
            let pack = load_pack_from_path_or_config(path)?;
            for descriptor in pack.descriptors {
                descriptors.insert(descriptor.candle_series_id.clone(), descriptor);
            }
        }
    }
    Ok(descriptors)
}

pub fn load_row_map_from_paths(
    paths: &[String],
) -> Result<BTreeMap<String, ComparableCommitteeEvidenceRow>, String> {
    let rows = load_rows_from_paths(paths)?;
    Ok(rows
        .into_iter()
        .map(|row| (row.row_id.clone(), row))
        .collect::<BTreeMap<_, _>>())
}

pub fn load_rows_from_paths(
    paths: &[String],
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let mut rows = Vec::new();
    for path in paths {
        if path.ends_with(".toml") {
            let config = ComparableCommitteeEvidenceConfig::from_toml_path(Path::new(path))?;
            rows.extend(ComparableEvidenceBuilder::default().build(&config)?.rows);
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
    Ok(rows)
}

pub fn load_future_bars_from_csv(path: &Path) -> Result<Vec<FutureBar>, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| format!("{} is empty", path.display()))?
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let timestamp_index = header
        .iter()
        .position(|value| value == "timestamp" || value == "timestamp_ms")
        .ok_or_else(|| format!("{} missing timestamp column", path.display()))?;
    let open_index = header
        .iter()
        .position(|value| value == "open")
        .ok_or_else(|| format!("{} missing open column", path.display()))?;
    let high_index = header
        .iter()
        .position(|value| value == "high")
        .ok_or_else(|| format!("{} missing high column", path.display()))?;
    let low_index = header
        .iter()
        .position(|value| value == "low")
        .ok_or_else(|| format!("{} missing low column", path.display()))?;
    let close_index = header
        .iter()
        .position(|value| value == "close")
        .ok_or_else(|| format!("{} missing close column", path.display()))?;
    let mut bars = Vec::new();
    for line in lines {
        let columns = line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        if columns.len() <= close_index {
            continue;
        }
        let Ok(timestamp_ms) = columns[timestamp_index].parse::<u64>() else {
            continue;
        };
        let Ok(open) = columns[open_index].parse::<f64>() else {
            continue;
        };
        let Ok(high) = columns[high_index].parse::<f64>() else {
            continue;
        };
        let Ok(low) = columns[low_index].parse::<f64>() else {
            continue;
        };
        let Ok(close) = columns[close_index].parse::<f64>() else {
            continue;
        };
        bars.push(FutureBar {
            timestamp_ms: normalize_timestamp(timestamp_ms),
            open,
            high,
            low,
            close,
        });
    }
    bars.sort_by_key(|bar| bar.timestamp_ms);
    Ok(bars)
}

fn build_requirement_item(
    config: &FutureWindowRequirementConfig,
    item: &OfficialReadyRowInventoryItem,
    descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> FutureWindowRequirementItem {
    let mut reason_codes = item.reason_codes.clone();
    let horizon_bars = if item.horizon_bars > 0 {
        item.horizon_bars
    } else {
        config.default_horizon_bars
    };
    let step_ms = timeframe_seconds(&item.timeframe).map(|seconds| seconds.saturating_mul(1000));
    let required_future_bars = horizon_bars;
    let required_start_timestamp_ms = step_ms
        .map(|step| item.timestamp_ms.saturating_add(step))
        .unwrap_or(item.timestamp_ms);
    let required_end_timestamp_ms = step_ms
        .map(|step| {
            item.timestamp_ms
                .saturating_add(step.saturating_mul(horizon_bars as u64))
        })
        .unwrap_or(item.timestamp_ms);
    let candidate = choose_descriptor(item, descriptors);
    let can_extend_from_existing_csv = candidate.is_some();
    let can_extend_from_provider_collection = provider_extension_allowed(item, config);
    let (gap_kind, current_available_future_bars, missing_future_bars, extra_reasons) =
        determine_gap(
            config,
            item,
            candidate.as_ref(),
            horizon_bars,
            required_end_timestamp_ms,
            step_ms,
        );
    reason_codes.extend(extra_reasons);

    FutureWindowRequirementItem {
        row_id: item.row_id.clone(),
        scenario_row_id: item.scenario_row_id.clone(),
        comparable_row_id: item.comparable_row_id.clone(),
        candle_series_id: candidate
            .as_ref()
            .map(|descriptor| descriptor.candle_series_id.clone())
            .or_else(|| item.candle_series_id.clone()),
        symbol: item.symbol.clone(),
        market: item.market,
        venue: item.venue.clone(),
        timeframe: normalize_timeframe_label(&item.timeframe),
        timestamp_ms: item.timestamp_ms,
        horizon_bars,
        current_available_future_bars,
        required_future_bars,
        missing_future_bars,
        required_start_timestamp_ms,
        required_end_timestamp_ms,
        can_extend_from_existing_csv,
        can_extend_from_provider_collection,
        gap_kind,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn determine_gap(
    config: &FutureWindowRequirementConfig,
    item: &OfficialReadyRowInventoryItem,
    descriptor: Option<&OfficialCandleSeriesDescriptor>,
    horizon_bars: usize,
    required_end_timestamp_ms: u64,
    step_ms: Option<u64>,
) -> (FutureWindowGapKind, usize, usize, Vec<ReasonCode>) {
    let mut reasons = Vec::new();
    let source_class = item.source_class;
    if item.timestamp_ms == 0 {
        reasons.push(ReasonCode::UnsupportedTimestampFormat);
        return (
            FutureWindowGapKind::MissingScenarioTimestamp,
            0,
            horizon_bars,
            reasons,
        );
    }
    if horizon_bars == 0 {
        reasons.push(ReasonCode::FeatureUnavailable);
        return (FutureWindowGapKind::MissingHorizon, 0, 0, reasons);
    }
    if item.horizon_bars > 0
        && config.default_horizon_bars > 0
        && item.horizon_bars != config.default_horizon_bars
    {
        reasons.push(ReasonCode::HorizonFiltered);
        return (
            FutureWindowGapKind::HorizonMismatch,
            0,
            horizon_bars,
            reasons,
        );
    }
    if config.require_no_lookahead_safe && !item.no_lookahead_safe {
        reasons.push(ReasonCode::RejectedNoLookaheadReference);
        return (
            FutureWindowGapKind::NoLookaheadViolation,
            0,
            horizon_bars,
            reasons,
        );
    }
    match source_class {
        ComparableEvidenceSourceClass::ControlledDiagnostic => {
            if !config.allow_controlled_diagnostic {
                reasons.push(ReasonCode::ControlledOnlyEvidence);
                return (
                    FutureWindowGapKind::SourceIneligible,
                    0,
                    horizon_bars,
                    reasons,
                );
            }
            reasons.push(ReasonCode::ControlledOnlyEvidence);
            return (
                FutureWindowGapKind::DiagnosticOnly,
                0,
                horizon_bars,
                reasons,
            );
        }
        ComparableEvidenceSourceClass::OfficialCryptoOnly => {
            if !config.allow_crypto_only {
                reasons.push(ReasonCode::CryptoOnlyEvidence);
                return (
                    FutureWindowGapKind::SourceIneligible,
                    0,
                    horizon_bars,
                    reasons,
                );
            }
            reasons.push(ReasonCode::CryptoOnlyEvidence);
            return (
                FutureWindowGapKind::DiagnosticOnly,
                0,
                horizon_bars,
                reasons,
            );
        }
        ComparableEvidenceSourceClass::YFinanceResearch => {
            if !config.allow_yfinance_research {
                reasons.push(ReasonCode::YFinanceResearchOnly);
                return (
                    FutureWindowGapKind::SourceIneligible,
                    0,
                    horizon_bars,
                    reasons,
                );
            }
            reasons.push(ReasonCode::YFinanceResearchOnly);
            return (
                FutureWindowGapKind::DiagnosticOnly,
                0,
                horizon_bars,
                reasons,
            );
        }
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => {
            if !config.allow_fixture {
                reasons.push(ReasonCode::SyntheticFixtureEvidence);
                return (
                    FutureWindowGapKind::SourceIneligible,
                    0,
                    horizon_bars,
                    reasons,
                );
            }
            reasons.push(ReasonCode::SyntheticFixtureEvidence);
            return (
                FutureWindowGapKind::DiagnosticOnly,
                0,
                horizon_bars,
                reasons,
            );
        }
        ComparableEvidenceSourceClass::OfficialNonCrypto
        | ComparableEvidenceSourceClass::Unknown => {}
    }
    let Some(descriptor) = descriptor else {
        reasons.push(ReasonCode::MissingOfficialCandles);
        return (
            FutureWindowGapKind::MissingCandleWindow,
            0,
            horizon_bars,
            reasons,
        );
    };
    if normalize_symbol(&descriptor.symbol) != normalize_symbol(&item.symbol) {
        reasons.push(ReasonCode::InvalidSymbol);
        return (
            FutureWindowGapKind::SymbolMismatch,
            0,
            horizon_bars,
            reasons,
        );
    }
    if normalize_timeframe_label(&descriptor.timeframe)
        != normalize_timeframe_label(&item.timeframe)
    {
        reasons.push(ReasonCode::UnsupportedTimeframe);
        return (
            FutureWindowGapKind::TimeframeMismatch,
            0,
            horizon_bars,
            reasons,
        );
    }
    let Some(step_ms) = step_ms else {
        reasons.push(ReasonCode::UnsupportedTimeframe);
        return (
            FutureWindowGapKind::TimeframeMismatch,
            0,
            horizon_bars,
            reasons,
        );
    };
    if item.timestamp_ms < descriptor.timestamp_start_ms
        || item.timestamp_ms >= descriptor.timestamp_end_ms
    {
        reasons.push(ReasonCode::UnsupportedTimestampFormat);
        return (
            FutureWindowGapKind::TimestampOutsideRange,
            0,
            horizon_bars,
            reasons,
        );
    }

    let timestamps = load_candle_csv_timestamp_series(Path::new(&descriptor.path))
        .map(|series| series.timestamps)
        .unwrap_or_default();
    if timestamps.is_empty() {
        reasons.push(ReasonCode::MissingRealLocalData);
        return (
            FutureWindowGapKind::MissingCandleWindow,
            0,
            horizon_bars,
            reasons,
        );
    }
    if !timestamps.contains(&item.timestamp_ms) {
        reasons.push(ReasonCode::UnsupportedTimestampFormat);
        return (
            FutureWindowGapKind::TimestampOutsideRange,
            0,
            horizon_bars,
            reasons,
        );
    }
    let available = contiguous_future_bars(&timestamps, item.timestamp_ms, step_ms, horizon_bars);
    let missing = horizon_bars.saturating_sub(available);
    if missing > 0 || descriptor.timestamp_end_ms < required_end_timestamp_ms {
        reasons.push(ReasonCode::InsufficientBars);
        return (
            FutureWindowGapKind::MissingFutureBars,
            available,
            missing.max(1),
            reasons,
        );
    }
    (
        FutureWindowGapKind::SufficientFutureBars,
        available,
        0,
        reasons,
    )
}

fn contiguous_future_bars(
    timestamps: &[u64],
    entry_timestamp_ms: u64,
    step_ms: u64,
    horizon_bars: usize,
) -> usize {
    let Some(entry_index) = timestamps
        .iter()
        .position(|value| *value == entry_timestamp_ms)
    else {
        return 0;
    };
    let mut expected = entry_timestamp_ms;
    let mut count = 0usize;
    for offset in 1..=horizon_bars {
        expected = expected.saturating_add(step_ms);
        let next_index = entry_index + offset;
        let Some(next) = timestamps.get(next_index) else {
            break;
        };
        if *next != expected {
            break;
        }
        count += 1;
    }
    count
}

fn determine_status(items: &[FutureWindowRequirementItem]) -> FutureWindowRequirementStatus {
    if items.is_empty() {
        return FutureWindowRequirementStatus::DiagnosticOnly;
    }
    if items
        .iter()
        .all(|item| item.gap_kind == FutureWindowGapKind::SufficientFutureBars)
    {
        return FutureWindowRequirementStatus::HealthyFutureWindows;
    }
    if items
        .iter()
        .all(|item| item.gap_kind == FutureWindowGapKind::SourceIneligible)
    {
        return FutureWindowRequirementStatus::SourceIneligible;
    }
    if items
        .iter()
        .all(|item| item.gap_kind == FutureWindowGapKind::DiagnosticOnly)
    {
        return FutureWindowRequirementStatus::DiagnosticOnly;
    }
    if items.iter().any(|item| {
        matches!(
            item.gap_kind,
            FutureWindowGapKind::MissingScenarioTimestamp
                | FutureWindowGapKind::TimestampOutsideRange
        )
    }) {
        return FutureWindowRequirementStatus::NeedTimestampAlignment;
    }
    if items.iter().any(|item| {
        matches!(
            item.gap_kind,
            FutureWindowGapKind::MissingHorizon | FutureWindowGapKind::HorizonMismatch
        )
    }) {
        return FutureWindowRequirementStatus::NeedHorizonAlignment;
    }
    if items.iter().any(|item| {
        item.gap_kind == FutureWindowGapKind::MissingFutureBars && item.can_extend_from_existing_csv
    }) {
        return FutureWindowRequirementStatus::NeedOfficialCandleExtension;
    }
    if items.iter().any(|item| {
        item.gap_kind == FutureWindowGapKind::MissingFutureBars
            && item.can_extend_from_provider_collection
    }) {
        return FutureWindowRequirementStatus::NeedProviderCollection;
    }
    FutureWindowRequirementStatus::NeedLongerFutureWindow
}

fn choose_descriptor(
    item: &OfficialReadyRowInventoryItem,
    descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> Option<OfficialCandleSeriesDescriptor> {
    if let Some(candle_series_id) = item.candle_series_id.as_ref() {
        if let Some(descriptor) = descriptors.get(candle_series_id) {
            return Some(descriptor.clone());
        }
    }
    let normalized_symbol = normalize_symbol(&item.symbol);
    descriptors
        .values()
        .filter(|descriptor| {
            normalize_symbol(&descriptor.symbol) == normalized_symbol
                && normalize_timeframe_label(&descriptor.timeframe)
                    == normalize_timeframe_label(&item.timeframe)
                && descriptor.market == item.market
        })
        .cloned()
        .max_by(|left, right| {
            left.timestamp_end_ms
                .cmp(&right.timestamp_end_ms)
                .then(left.row_count.cmp(&right.row_count))
                .then(left.candle_series_id.cmp(&right.candle_series_id))
        })
}

fn provider_extension_allowed(
    item: &OfficialReadyRowInventoryItem,
    config: &FutureWindowRequirementConfig,
) -> bool {
    match item.source_class {
        ComparableEvidenceSourceClass::OfficialNonCrypto => true,
        ComparableEvidenceSourceClass::OfficialCryptoOnly => config.allow_crypto_only,
        ComparableEvidenceSourceClass::ControlledDiagnostic => config.allow_controlled_diagnostic,
        ComparableEvidenceSourceClass::YFinanceResearch => config.allow_yfinance_research,
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => config.allow_fixture,
        ComparableEvidenceSourceClass::Unknown => false,
    }
}

fn load_inventory_path(path: &str) -> Result<OfficialReadyRowInventoryReport, String> {
    if path.ends_with(".json") {
        OfficialReadyRowInventoryReport::from_json_path(Path::new(path))
    } else {
        let config = OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
        OfficialReadyRowInventoryRunner::default().run(&config)
    }
}

pub fn descriptor_from_csv_path(path: &str) -> Result<OfficialCandleSeriesDescriptor, String> {
    let path_ref = Path::new(path);
    let series = load_candle_csv_timestamp_series(path_ref)?;
    let provenance_path = sidecar_path(path_ref, "provenance");
    let preflight_path = sidecar_path(path_ref, "preflight");
    let provenance = provenance_path
        .as_ref()
        .and_then(|path| read_json::<crate::data::DataProvenance>(path).ok());
    let preflight = preflight_path
        .as_ref()
        .and_then(|path| read_json::<PreflightReport>(path).ok());
    let source_kind = provenance
        .as_ref()
        .map(|record| record.source_kind)
        .unwrap_or(EvidenceSourceKind::OfficialApiCollected);
    let provider_kind = provenance
        .as_ref()
        .and_then(|record| record.provider_label.as_deref())
        .and_then(parse_provider_kind);
    let storage_bytes = fs::metadata(path_ref)
        .map(|metadata| metadata.len() as usize)
        .unwrap_or_default();
    let source_class = descriptor_source_class(source_kind, provider_kind);
    Ok(OfficialCandleSeriesDescriptor {
        candle_series_id: path_ref
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("candle-series")
            .to_string(),
        path: path.to_string(),
        provider_kind,
        source_kind,
        source_class,
        market: infer_market_from_descriptor(&series.symbol, source_class),
        venue: None,
        symbol: series.symbol.clone(),
        normalized_symbol: series.normalized_symbol.clone(),
        timeframe: series.timeframe.clone(),
        row_count: series.timestamps.len(),
        timestamp_start_ms: *series.timestamps.first().unwrap_or(&0),
        timestamp_end_ms: *series.timestamps.last().unwrap_or(&0),
        has_duplicates: series
            .timestamps
            .windows(2)
            .any(|window| window[0] == window[1]),
        has_gaps: series.timestamps.windows(2).any(|window| {
            window[1].saturating_sub(window[0])
                > timeframe_seconds(&series.timeframe)
                    .map(|seconds| seconds.saturating_mul(1000))
                    .unwrap_or(u64::MAX)
        }),
        data_quality_score: None,
        provenance_available: provenance_path.is_some(),
        preflight_ready: preflight.as_ref().is_some_and(|report| {
            report.final_status == PreflightFinalStatus::ReadyForRealEvidence
        }),
        manifest_available: false,
        timestamp_policy: series.timestamp_policy,
        adjusted_price_policy: None,
        official_readiness_eligible: matches!(
            source_class,
            OfficialCandleSeriesSourceClass::OfficialNonCrypto
        ),
        benchmark_eligible: !matches!(
            source_class,
            OfficialCandleSeriesSourceClass::YFinanceResearch
                | OfficialCandleSeriesSourceClass::FixtureArchitectureTest
                | OfficialCandleSeriesSourceClass::SyntheticTest
        ),
        diagnostic_only: matches!(
            source_class,
            OfficialCandleSeriesSourceClass::OfficialCryptoOnly
                | OfficialCandleSeriesSourceClass::ControlledDiagnostic
                | OfficialCandleSeriesSourceClass::YFinanceResearch
                | OfficialCandleSeriesSourceClass::FixtureArchitectureTest
                | OfficialCandleSeriesSourceClass::SyntheticTest
        ),
        storage_bytes,
        reason_codes: stable_reason_codes(&[
            ReasonCode::DeterministicPath,
            ReasonCode::LocalFileOnly,
        ]),
    })
}

fn sidecar_path(path: &Path, suffix: &str) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_str()?;
    let candidate = path.with_file_name(format!("{stem}_{suffix}.json"));
    candidate.exists().then_some(candidate)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn parse_provider_kind(value: &str) -> Option<ProviderKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "alphavantage" => Some(ProviderKind::AlphaVantage),
        "alpaca" => Some(ProviderKind::Alpaca),
        "upbit" => Some(ProviderKind::Upbit),
        "krxopenapi" | "krx" => Some(ProviderKind::KrxOpenApi),
        "datagokr" | "data-go-kr" => Some(ProviderKind::DataGoKrFscStockPrice),
        _ => None,
    }
}

fn descriptor_source_class(
    source_kind: EvidenceSourceKind,
    provider_kind: Option<ProviderKind>,
) -> OfficialCandleSeriesSourceClass {
    match source_kind {
        EvidenceSourceKind::OfficialApiCollected => match provider_kind {
            Some(ProviderKind::Upbit) => OfficialCandleSeriesSourceClass::OfficialCryptoOnly,
            _ => OfficialCandleSeriesSourceClass::OfficialNonCrypto,
        },
        EvidenceSourceKind::RealLocal => OfficialCandleSeriesSourceClass::ControlledDiagnostic,
        EvidenceSourceKind::YFinanceResearch => OfficialCandleSeriesSourceClass::YFinanceResearch,
        EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::GeneratedSynthetic => {
            OfficialCandleSeriesSourceClass::SyntheticTest
        }
        EvidenceSourceKind::TestFixture => OfficialCandleSeriesSourceClass::FixtureArchitectureTest,
        _ => OfficialCandleSeriesSourceClass::Unknown,
    }
}

fn infer_market_from_descriptor(
    symbol: &str,
    source_class: OfficialCandleSeriesSourceClass,
) -> ProviderMarket {
    if matches!(
        source_class,
        OfficialCandleSeriesSourceClass::OfficialCryptoOnly
    ) || symbol.contains('-')
        || symbol.contains("USDT")
    {
        ProviderMarket::Crypto
    } else {
        ProviderMarket::USEquity
    }
}

fn normalize_timestamp(value: u64) -> u64 {
    if value < 1_000_000_000_000 {
        value.saturating_mul(1000)
    } else {
        value
    }
}

fn input_storage_bytes(paths: &[String]) -> usize {
    paths
        .iter()
        .filter_map(|path| fs::metadata(path).ok())
        .map(|metadata| metadata.len() as usize)
        .sum()
}

fn default_output_root() -> String {
    "target/soma_future_window_requirements".to_string()
}

fn default_horizon_bars() -> usize {
    24
}

fn default_take_profit_pct() -> f64 {
    0.02
}

fn default_stop_loss_pct() -> f64 {
    0.01
}

fn default_cost_bps() -> f64 {
    5.0
}

fn default_slippage_bps() -> f64 {
    2.0
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
