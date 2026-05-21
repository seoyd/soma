use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::counterfactual_completion_v2::{
    CounterfactualCompletionRecord, load_counterfactual_completion_v2_from_path_or_config,
};
use super::future_window_requirements::{load_descriptor_map_from_paths, load_rows_from_paths};
use super::future_window_scaleout::load_future_window_scaleout_plan_from_path_or_config;
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryConfig, OfficialReadyRowInventoryItem,
    OfficialReadyRowInventoryReport, OfficialReadyRowInventoryRunner,
};
use super::outcome_linkage_v3::{
    OutcomeLinkageV3Record, load_outcome_linkage_v3_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiRowOfficialEvidenceSetConfig {
    pub set_id: String,
    #[serde(default)]
    pub official_ready_inventory_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_closure_v2_paths: Vec<String>,
    #[serde(default)]
    pub official_ready_match_closure_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_coverage_pack_paths: Vec<String>,
    #[serde(default)]
    pub future_window_requirement_paths: Vec<String>,
    #[serde(default)]
    pub future_window_extension_plan_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_pack_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub official_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub provenance_paths: Vec<String>,
    #[serde(default)]
    pub preflight_report_paths: Vec<String>,
    #[serde(default)]
    pub outcome_linkage_v3_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_completion_v2_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_completion_paths: Vec<String>,
    #[serde(default)]
    pub include_row_ids: Vec<String>,
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
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_non_crypto_official: bool,
    #[serde(default = "default_true")]
    pub require_official_ready_match: bool,
    #[serde(default = "default_true")]
    pub require_future_window_sufficient: bool,
    #[serde(default = "default_true")]
    pub require_outcome_reference: bool,
    #[serde(default = "default_true")]
    pub require_baseline_reference: bool,
    #[serde(default = "default_true")]
    pub require_no_trade_counterfactual: bool,
    #[serde(default = "default_true")]
    pub require_risk_denied_counterfactual: bool,
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
pub enum MultiRowOfficialEvidenceStatus {
    OfficialComplete,
    OfficialPartialMissingFutureWindow,
    OfficialPartialMissingOutcome,
    OfficialPartialMissingBaseline,
    OfficialPartialMissingNoTradeCounterfactual,
    OfficialPartialMissingRiskDeniedCounterfactual,
    OfficialPartialMissingCommitteeDecision,
    OfficialPartialMissingRiskDecision,
    DiagnosticControlled,
    CryptoOnly,
    ResearchOnly,
    FixtureOnly,
    #[default]
    SourceIneligible,
    NoLookaheadRejected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MultiRowOfficialEvidenceItem {
    pub row_id: String,
    #[serde(default)]
    pub scenario_row_id: Option<String>,
    #[serde(default)]
    pub comparable_row_id: Option<String>,
    #[serde(default)]
    pub candle_series_id: Option<String>,
    pub symbol: String,
    pub market: crate::data::ProviderMarket,
    #[serde(default)]
    pub venue: Option<String>,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub timestamp_ms: u64,
    pub source_kind: String,
    pub source_class: ComparableEvidenceSourceClass,
    pub official_ready_match: bool,
    pub future_window_sufficient: bool,
    pub outcome_reference_available: bool,
    pub baseline_reference_available: bool,
    pub no_trade_counterfactual_available: bool,
    pub risk_denied_counterfactual_available: bool,
    pub committee_decision_available: bool,
    pub risk_decision_available: bool,
    pub no_lookahead_safe: bool,
    #[serde(default)]
    pub net_return_pct: Option<f64>,
    #[serde(default)]
    pub no_trade_value_proxy: Option<f64>,
    #[serde(default)]
    pub risk_denied_value_proxy: Option<f64>,
    pub status: MultiRowOfficialEvidenceStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    #[serde(default, skip_serializing)]
    pub benchmark_ready_match: bool,
    #[serde(default, skip_serializing)]
    pub row_level: bool,
    #[serde(default, skip_serializing)]
    pub summary_derived: bool,
    #[serde(default, skip_serializing)]
    pub has_provenance: bool,
    #[serde(default, skip_serializing)]
    pub has_preflight: bool,
    #[serde(default, skip_serializing)]
    pub has_local_candle_window: bool,
    #[serde(default, skip_serializing)]
    pub official_complete: bool,
    #[serde(default, skip_serializing)]
    pub diagnostic_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MultiRowOfficialEvidenceSet {
    pub set_id: String,
    pub items: Vec<MultiRowOfficialEvidenceItem>,
    pub total_rows: usize,
    pub official_complete_rows: usize,
    pub official_partial_rows: usize,
    pub non_crypto_official_rows: usize,
    pub crypto_only_rows: usize,
    pub controlled_rows: usize,
    pub yfinance_rows: usize,
    pub fixture_rows: usize,
    pub outcome_reference_count: usize,
    pub baseline_reference_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub no_lookahead_safe_count: usize,
    pub storage_bytes: usize,
    #[serde(default, skip_serializing)]
    pub symbol_count: usize,
    #[serde(default, skip_serializing)]
    pub timeframe_count: usize,
    #[serde(default, skip_serializing)]
    pub horizon_count: usize,
    #[serde(default, skip_serializing)]
    pub source_boundaries_preserved: bool,
    #[serde(default)]
    pub status: MultiRowOfficialEvidenceStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MultiRowOfficialEvidenceSetBuilder;

impl Default for MultiRowOfficialEvidenceSetConfig {
    fn default() -> Self {
        Self {
            set_id: "multi-row-official-evidence".to_string(),
            official_ready_inventory_paths: Vec::new(),
            complete_row_closure_v2_paths: Vec::new(),
            official_ready_match_closure_paths: Vec::new(),
            official_candle_coverage_pack_paths: Vec::new(),
            future_window_requirement_paths: Vec::new(),
            future_window_extension_plan_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            official_candle_pack_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            canonical_csv_paths: Vec::new(),
            official_canonical_csv_paths: Vec::new(),
            provenance_paths: Vec::new(),
            preflight_report_paths: Vec::new(),
            outcome_linkage_v3_paths: Vec::new(),
            counterfactual_completion_v2_paths: Vec::new(),
            counterfactual_completion_paths: Vec::new(),
            include_row_ids: Vec::new(),
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_timeframes: default_max_timeframes(),
            max_horizons: default_max_horizons(),
            max_bytes: default_max_bytes(),
            require_non_crypto_official: true,
            require_official_ready_match: true,
            require_future_window_sufficient: true,
            require_outcome_reference: true,
            require_baseline_reference: true,
            require_no_trade_counterfactual: true,
            require_risk_denied_counterfactual: true,
            require_no_lookahead_safe: true,
            allow_controlled_diagnostic: false,
            allow_crypto_only: true,
            allow_yfinance_research: false,
            allow_fixture: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl MultiRowOfficialEvidenceSetConfig {
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
        if self.set_id.trim().is_empty() {
            return Err("multi-row official evidence set id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("multi-row official evidence set paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "multi-row official evidence set max_rows must be between 1 and 1000".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "multi-row official evidence set max_symbols must be between 1 and 10".to_string(),
            );
        }
        if self.max_timeframes == 0 || self.max_timeframes > default_max_timeframes() {
            return Err(
                "multi-row official evidence set max_timeframes must be between 1 and 5"
                    .to_string(),
            );
        }
        if self.max_horizons == 0 || self.max_horizons > default_max_horizons() {
            return Err(
                "multi-row official evidence set max_horizons must be between 1 and 5".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "multi-row official evidence set max_bytes must be between 1 and 5000000"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.set_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.official_ready_inventory_paths
            .iter()
            .chain(self.complete_row_closure_v2_paths.iter())
            .chain(self.official_ready_match_closure_paths.iter())
            .chain(self.official_candle_coverage_pack_paths.iter())
            .chain(self.future_window_requirement_paths.iter())
            .chain(self.future_window_extension_plan_paths.iter())
            .chain(self.comparable_evidence_bundle_paths.iter())
            .chain(self.official_candle_pack_paths.iter())
            .chain(self.core_scorecard_paths.iter())
            .chain(self.canonical_csv_paths.iter())
            .chain(self.official_canonical_csv_paths.iter())
            .chain(self.provenance_paths.iter())
            .chain(self.preflight_report_paths.iter())
            .chain(self.outcome_linkage_v3_paths.iter())
            .chain(self.counterfactual_completion_v2_paths.iter())
            .chain(self.counterfactual_completion_paths.iter())
            .cloned()
            .collect()
    }

    fn candle_paths(&self) -> Vec<String> {
        self.official_candle_coverage_pack_paths
            .iter()
            .chain(self.official_candle_pack_paths.iter())
            .chain(self.canonical_csv_paths.iter())
            .chain(self.official_canonical_csv_paths.iter())
            .cloned()
            .collect()
    }

    fn counterfactual_paths(&self) -> Vec<String> {
        self.counterfactual_completion_v2_paths
            .iter()
            .chain(self.counterfactual_completion_paths.iter())
            .cloned()
            .collect()
    }
}

impl MultiRowOfficialEvidenceSetBuilder {
    pub fn build(
        &self,
        config: &MultiRowOfficialEvidenceSetConfig,
    ) -> Result<MultiRowOfficialEvidenceSet, String> {
        config.validate()?;
        let descriptors = load_descriptor_map_from_paths(&config.candle_paths())?;
        let row_paths = if config.comparable_evidence_bundle_paths.is_empty() {
            config.complete_row_closure_v2_paths.clone()
        } else {
            config.comparable_evidence_bundle_paths.clone()
        };
        let rows = load_rows_from_paths(&row_paths)?;
        let row_map = rows
            .iter()
            .cloned()
            .map(|row| (row.row_id.clone(), row))
            .collect::<BTreeMap<_, _>>();
        let inventory = load_inventory(config, &rows, &descriptors)?;
        let future_window_map = load_future_window_map(
            &config.future_window_requirement_paths,
            &config.future_window_extension_plan_paths,
        )?;
        let outcome_map = load_outcome_map(&config.outcome_linkage_v3_paths)?;
        let counterfactual_map = load_counterfactual_map(&config.counterfactual_paths())?;
        let provenance_map = load_provenance_map(&config.provenance_paths)?;
        let preflight_map = load_preflight_map(&config.preflight_report_paths)?;
        let allowed_rows = config
            .include_row_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let filter_row = |row_id: &str| allowed_rows.is_empty() || allowed_rows.contains(row_id);

        let mut items = inventory
            .items
            .iter()
            .filter(|item| filter_row(&item.row_id))
            .filter(|item| source_allowed(item.source_class, config))
            .map(|item| {
                build_item(
                    item,
                    row_map.get(&item.row_id),
                    outcome_map.get(&item.row_id),
                    counterfactual_map.get(&item.row_id),
                    future_window_map.get(&item.row_id).copied(),
                    &descriptors,
                    &provenance_map,
                    &preflight_map,
                    config,
                )
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.row_id
                .cmp(&right.row_id)
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timestamp_ms.cmp(&right.timestamp_ms))
                .then(left.timeframe.cmp(&right.timeframe))
        });

        if items.len() > config.max_rows {
            return Err(format!(
                "multi-row official evidence set loaded {} rows which exceeds max_rows {}",
                items.len(),
                config.max_rows
            ));
        }
        let symbol_count = items
            .iter()
            .map(|item| item.symbol.clone())
            .collect::<BTreeSet<_>>()
            .len();
        if symbol_count > config.max_symbols {
            return Err(format!(
                "multi-row official evidence set loaded {} symbols which exceeds max_symbols {}",
                symbol_count, config.max_symbols
            ));
        }
        let timeframe_count = items
            .iter()
            .map(|item| item.timeframe.clone())
            .collect::<BTreeSet<_>>()
            .len();
        if timeframe_count > config.max_timeframes {
            return Err(format!(
                "multi-row official evidence set loaded {} timeframes which exceeds max_timeframes {}",
                timeframe_count, config.max_timeframes
            ));
        }
        let horizon_count = items
            .iter()
            .map(|item| item.horizon_bars)
            .collect::<BTreeSet<_>>()
            .len();
        if horizon_count > config.max_horizons {
            return Err(format!(
                "multi-row official evidence set loaded {} horizons which exceeds max_horizons {}",
                horizon_count, config.max_horizons
            ));
        }

        let total_rows = items.len();
        let official_complete_rows = items.iter().filter(|item| item.official_complete).count();
        let official_partial_rows = items
            .iter()
            .filter(|item| {
                item.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto
                    && !item.official_complete
            })
            .count();
        let non_crypto_official_rows = items
            .iter()
            .filter(|item| item.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
            .count();
        let crypto_only_rows = items
            .iter()
            .filter(|item| item.source_class == ComparableEvidenceSourceClass::OfficialCryptoOnly)
            .count();
        let controlled_rows = items
            .iter()
            .filter(|item| item.source_class == ComparableEvidenceSourceClass::ControlledDiagnostic)
            .count();
        let yfinance_rows = items
            .iter()
            .filter(|item| item.source_class == ComparableEvidenceSourceClass::YFinanceResearch)
            .count();
        let fixture_rows = items
            .iter()
            .filter(|item| {
                matches!(
                    item.source_class,
                    ComparableEvidenceSourceClass::FixtureArchitectureTest
                        | ComparableEvidenceSourceClass::SyntheticTest
                )
            })
            .count();
        let outcome_reference_count = items
            .iter()
            .filter(|item| item.outcome_reference_available)
            .count();
        let baseline_reference_count = items
            .iter()
            .filter(|item| item.baseline_reference_available)
            .count();
        let no_trade_counterfactual_count = items
            .iter()
            .filter(|item| item.no_trade_counterfactual_available)
            .count();
        let risk_denied_counterfactual_count = items
            .iter()
            .filter(|item| item.risk_denied_counterfactual_available)
            .count();
        let no_lookahead_safe_count = items.iter().filter(|item| item.no_lookahead_safe).count();
        let source_boundaries_preserved = items.iter().all(|item| match item.source_class {
            ComparableEvidenceSourceClass::OfficialNonCrypto => !item.diagnostic_only,
            ComparableEvidenceSourceClass::OfficialCryptoOnly => config.allow_crypto_only,
            ComparableEvidenceSourceClass::ControlledDiagnostic => {
                config.allow_controlled_diagnostic
            }
            ComparableEvidenceSourceClass::YFinanceResearch => config.allow_yfinance_research,
            ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest => config.allow_fixture,
            ComparableEvidenceSourceClass::Unknown => false,
        });
        let storage_bytes = serde_json::to_vec(&items)
            .map(|bytes| bytes.len())
            .unwrap_or_default();
        if storage_bytes > config.max_bytes {
            return Err(format!(
                "multi-row official evidence set estimated {} bytes which exceeds max_bytes {}",
                storage_bytes, config.max_bytes
            ));
        }
        let warnings = build_warnings(&items, official_complete_rows);
        let status = determine_status(&items, official_complete_rows, source_boundaries_preserved);
        Ok(MultiRowOfficialEvidenceSet {
            set_id: config.set_id.clone(),
            items,
            total_rows,
            official_complete_rows,
            official_partial_rows,
            non_crypto_official_rows,
            crypto_only_rows,
            controlled_rows,
            yfinance_rows,
            fixture_rows,
            outcome_reference_count,
            baseline_reference_count,
            no_trade_counterfactual_count,
            risk_denied_counterfactual_count,
            no_lookahead_safe_count,
            storage_bytes,
            symbol_count,
            timeframe_count,
            horizon_count,
            source_boundaries_preserved,
            status,
            warnings,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::OfficialEvidenceCounted,
                        ReasonCode::DeterministicPath,
                        ReasonCode::LocalFileOnly,
                    ])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl MultiRowOfficialEvidenceSet {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.set_id.clone()))
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("set_id={}", self.set_id),
            format!("total_rows={}", self.total_rows),
            format!("official_complete_rows={}", self.official_complete_rows),
            format!("official_partial_rows={}", self.official_partial_rows),
            format!("non_crypto_official_rows={}", self.non_crypto_official_rows),
            format!("crypto_only_rows={}", self.crypto_only_rows),
            format!("controlled_rows={}", self.controlled_rows),
            format!("yfinance_rows={}", self.yfinance_rows),
            format!("fixture_rows={}", self.fixture_rows),
            format!("outcome_reference_count={}", self.outcome_reference_count),
            format!("baseline_reference_count={}", self.baseline_reference_count),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denied_counterfactual_count={}",
                self.risk_denied_counterfactual_count
            ),
            format!("no_lookahead_safe_count={}", self.no_lookahead_safe_count),
            format!("storage_bytes={}", self.storage_bytes),
            format!("symbol_count={}", self.symbol_count),
            format!("timeframe_count={}", self.timeframe_count),
            format!("horizon_count={}", self.horizon_count),
            format!(
                "source_boundaries_preserved={}",
                self.source_boundaries_preserved
            ),
            format!("status={:?}", self.status),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "row_id={};symbol={};timeframe={};horizon_bars={};source_class={:?};status={:?};future_window_sufficient={};outcome_reference_available={};baseline_reference_available={};no_trade_counterfactual_available={};risk_denied_counterfactual_available={};committee_decision_available={};risk_decision_available={};net_return_pct={};no_trade_value_proxy={};risk_denied_value_proxy={}",
                item.row_id,
                item.symbol,
                item.timeframe,
                item.horizon_bars,
                item.source_class,
                item.status,
                item.future_window_sufficient,
                item.outcome_reference_available,
                item.baseline_reference_available,
                item.no_trade_counterfactual_available,
                item.risk_denied_counterfactual_available,
                item.committee_decision_available,
                item.risk_decision_available,
                item.net_return_pct.map(|value| value.to_string()).unwrap_or_default(),
                item.no_trade_value_proxy
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                item.risk_denied_value_proxy
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
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
            output_dir.join("multi_row_official_evidence_set.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("multi_row_official_evidence_set.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_multi_row_official_evidence_set_from_path_or_config(
    path: &str,
) -> Result<MultiRowOfficialEvidenceSet, String> {
    if path.ends_with(".json") {
        MultiRowOfficialEvidenceSet::from_json_path(Path::new(path))
    } else {
        MultiRowOfficialEvidenceSetConfig::from_toml_path(Path::new(path))
            .and_then(|config| MultiRowOfficialEvidenceSetBuilder::default().build(&config))
    }
}

fn build_item(
    inventory_item: &OfficialReadyRowInventoryItem,
    row: Option<&ComparableCommitteeEvidenceRow>,
    outcome_record: Option<&OutcomeLinkageV3Record>,
    counterfactual_record: Option<&CounterfactualCompletionRecord>,
    future_window_sufficient: Option<bool>,
    descriptors: &BTreeMap<
        String,
        super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
    >,
    provenance_map: &BTreeMap<String, bool>,
    preflight_map: &BTreeMap<String, bool>,
    config: &MultiRowOfficialEvidenceSetConfig,
) -> MultiRowOfficialEvidenceItem {
    let outcome_reference_available = outcome_record
        .and_then(|record| record.outcome_reference.as_ref())
        .is_some()
        || inventory_item.has_outcome_reference
        || row.is_some_and(|row| row.outcome_reference_available);
    let baseline_reference_available = inventory_item.has_baseline_reference
        || row.is_some_and(|row| row.baseline_reference_available);
    let no_trade_counterfactual_available = counterfactual_record
        .map(|record| record.no_trade_counterfactual_built)
        .unwrap_or(false)
        || inventory_item.has_no_trade_counterfactual
        || row.is_some_and(|row| row.no_trade_counterfactual_available);
    let risk_denied_counterfactual_available = counterfactual_record
        .map(|record| record.risk_denied_counterfactual_built)
        .unwrap_or(false)
        || inventory_item.has_risk_denied_counterfactual
        || row.is_some_and(|row| row.risk_denied_counterfactual_available);
    let committee_decision_available = inventory_item.has_committee_decision
        || row.is_some_and(|row| !row.committee_final_action.trim().is_empty());
    let risk_decision_available = inventory_item.has_risk_decision
        || row.is_some_and(|row| {
            row.risk_governor_decision
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        });
    let key = item_key(
        inventory_item.candle_series_id.as_deref(),
        &inventory_item.symbol,
        &inventory_item.timeframe,
    );
    let has_local_candle_window = inventory_item
        .candle_series_id
        .as_ref()
        .and_then(|id| descriptors.get(id))
        .is_some()
        || descriptors
            .values()
            .any(|descriptor| descriptor_key(descriptor) == key);
    let has_provenance = provenance_map.get(&key).copied().unwrap_or(false);
    let has_preflight = preflight_map.get(&key).copied().unwrap_or(false);
    let future_window_sufficient = future_window_sufficient
        .unwrap_or(has_local_candle_window && has_provenance && has_preflight);
    let diagnostic_only = row.is_some_and(|row| row.diagnostic_only)
        || matches!(
            inventory_item.source_class,
            ComparableEvidenceSourceClass::ControlledDiagnostic
                | ComparableEvidenceSourceClass::YFinanceResearch
                | ComparableEvidenceSourceClass::FixtureArchitectureTest
                | ComparableEvidenceSourceClass::SyntheticTest
                | ComparableEvidenceSourceClass::OfficialCryptoOnly
        );
    let official_complete = (!config.require_non_crypto_official
        || inventory_item.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
        && (!config.require_official_ready_match || inventory_item.official_ready_match)
        && (!config.require_future_window_sufficient || future_window_sufficient)
        && (!config.require_no_lookahead_safe || inventory_item.no_lookahead_safe)
        && inventory_item.row_level
        && !inventory_item.summary_derived
        && (!config.require_outcome_reference || outcome_reference_available)
        && (!config.require_baseline_reference || baseline_reference_available)
        && (!config.require_no_trade_counterfactual || no_trade_counterfactual_available)
        && (!config.require_risk_denied_counterfactual || risk_denied_counterfactual_available)
        && committee_decision_available
        && risk_decision_available
        && has_local_candle_window
        && has_provenance
        && has_preflight;
    let status = determine_item_status(
        inventory_item,
        future_window_sufficient,
        outcome_reference_available,
        baseline_reference_available,
        no_trade_counterfactual_available,
        risk_denied_counterfactual_available,
        committee_decision_available,
        risk_decision_available,
        official_complete,
        config,
    );
    MultiRowOfficialEvidenceItem {
        row_id: inventory_item.row_id.clone(),
        scenario_row_id: inventory_item.scenario_row_id.clone(),
        comparable_row_id: inventory_item.comparable_row_id.clone(),
        candle_series_id: inventory_item.candle_series_id.clone(),
        symbol: inventory_item.symbol.clone(),
        market: inventory_item.market,
        venue: inventory_item.venue.clone(),
        timeframe: inventory_item.timeframe.clone(),
        horizon_bars: row
            .map(|row| row.horizon_bars)
            .unwrap_or(inventory_item.horizon_bars),
        timestamp_ms: inventory_item.timestamp_ms,
        source_kind: inventory_item.source_kind.clone(),
        source_class: inventory_item.source_class,
        official_ready_match: inventory_item.official_ready_match,
        future_window_sufficient,
        outcome_reference_available,
        baseline_reference_available,
        no_trade_counterfactual_available,
        risk_denied_counterfactual_available,
        committee_decision_available,
        risk_decision_available,
        no_lookahead_safe: inventory_item.no_lookahead_safe,
        net_return_pct: outcome_record
            .and_then(|record| record.net_return_pct)
            .or_else(|| row.and_then(|row| row.net_return_pct)),
        no_trade_value_proxy: counterfactual_record
            .and_then(|record| record.missed_gain_value)
            .or_else(|| row.and_then(|row| row.no_trade_value_proxy)),
        risk_denied_value_proxy: counterfactual_record
            .and_then(|record| record.avoided_loss_value)
            .or_else(|| row.and_then(|row| row.risk_denied_value_proxy)),
        status,
        reason_codes: stable_reason_codes(
            &inventory_item
                .reason_codes
                .iter()
                .cloned()
                .chain(row.into_iter().flat_map(|row| row.reason_codes.clone()))
                .chain(
                    outcome_record
                        .into_iter()
                        .flat_map(|record| record.reason_codes.clone()),
                )
                .chain(
                    counterfactual_record
                        .into_iter()
                        .flat_map(|record| record.reason_codes.clone()),
                )
                .collect::<Vec<_>>(),
        ),
        benchmark_ready_match: inventory_item.benchmark_ready_match,
        row_level: inventory_item.row_level,
        summary_derived: inventory_item.summary_derived,
        has_provenance,
        has_preflight,
        has_local_candle_window,
        official_complete,
        diagnostic_only,
    }
}

fn build_warnings(
    items: &[MultiRowOfficialEvidenceItem],
    official_complete_rows: usize,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if official_complete_rows <= 1 {
        warnings.push(
            "one or zero official complete rows remain insufficient for usefulness claims"
                .to_string(),
        );
    }
    if items
        .iter()
        .map(|item| item.symbol.clone())
        .collect::<BTreeSet<_>>()
        .len()
        <= 1
    {
        warnings.push("symbol diversity remains limited".to_string());
    }
    if items
        .iter()
        .map(|item| item.timeframe.clone())
        .collect::<BTreeSet<_>>()
        .len()
        <= 1
    {
        warnings.push("timeframe diversity remains limited".to_string());
    }
    warnings
}

fn determine_status(
    items: &[MultiRowOfficialEvidenceItem],
    official_complete_rows: usize,
    source_boundaries_preserved: bool,
) -> MultiRowOfficialEvidenceStatus {
    if !source_boundaries_preserved {
        return MultiRowOfficialEvidenceStatus::SourceIneligible;
    }
    if items.is_empty() {
        return MultiRowOfficialEvidenceStatus::SourceIneligible;
    }
    if official_complete_rows > 0 {
        return MultiRowOfficialEvidenceStatus::OfficialComplete;
    }
    for status in [
        MultiRowOfficialEvidenceStatus::NoLookaheadRejected,
        MultiRowOfficialEvidenceStatus::OfficialPartialMissingFutureWindow,
        MultiRowOfficialEvidenceStatus::OfficialPartialMissingOutcome,
        MultiRowOfficialEvidenceStatus::OfficialPartialMissingBaseline,
        MultiRowOfficialEvidenceStatus::OfficialPartialMissingNoTradeCounterfactual,
        MultiRowOfficialEvidenceStatus::OfficialPartialMissingRiskDeniedCounterfactual,
        MultiRowOfficialEvidenceStatus::OfficialPartialMissingCommitteeDecision,
        MultiRowOfficialEvidenceStatus::OfficialPartialMissingRiskDecision,
        MultiRowOfficialEvidenceStatus::DiagnosticControlled,
        MultiRowOfficialEvidenceStatus::CryptoOnly,
        MultiRowOfficialEvidenceStatus::ResearchOnly,
        MultiRowOfficialEvidenceStatus::FixtureOnly,
    ] {
        if items.iter().any(|item| item.status == status) {
            return status;
        }
    }
    MultiRowOfficialEvidenceStatus::SourceIneligible
}

fn source_allowed(
    source_class: ComparableEvidenceSourceClass,
    config: &MultiRowOfficialEvidenceSetConfig,
) -> bool {
    match source_class {
        ComparableEvidenceSourceClass::OfficialNonCrypto => true,
        ComparableEvidenceSourceClass::OfficialCryptoOnly => config.allow_crypto_only,
        ComparableEvidenceSourceClass::ControlledDiagnostic => config.allow_controlled_diagnostic,
        ComparableEvidenceSourceClass::YFinanceResearch => config.allow_yfinance_research,
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => config.allow_fixture,
        ComparableEvidenceSourceClass::Unknown => false,
    }
}

fn load_inventory(
    config: &MultiRowOfficialEvidenceSetConfig,
    rows: &[ComparableCommitteeEvidenceRow],
    descriptors: &BTreeMap<
        String,
        super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
    >,
) -> Result<OfficialReadyRowInventoryReport, String> {
    if let Some(path) = config.official_ready_inventory_paths.first() {
        return load_inventory_path(path);
    }
    let inventory_config = OfficialReadyRowInventoryConfig {
        inventory_id: format!("{}-inventory", config.set_id),
        output_root: config.output_root.clone(),
        max_rows: config.max_rows.min(500),
        max_symbols: config.max_symbols.min(5),
        require_official_ready_match: config.require_official_ready_match,
        require_no_lookahead_safe: config.require_no_lookahead_safe,
        allow_controlled_diagnostic: config.allow_controlled_diagnostic,
        allow_crypto_only: config.allow_crypto_only,
        allow_yfinance_research: config.allow_yfinance_research,
        allow_fixture: config.allow_fixture,
        reason_codes: config.reason_codes.clone(),
        ..OfficialReadyRowInventoryConfig::default()
    };
    OfficialReadyRowInventoryRunner::default().run_from_rows(
        &inventory_config,
        rows,
        &BTreeMap::new(),
        descriptors,
    )
}

fn load_inventory_path(path: &str) -> Result<OfficialReadyRowInventoryReport, String> {
    if path.ends_with(".json") {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    } else {
        let config = OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
        OfficialReadyRowInventoryRunner::default().run(&config)
    }
}

fn load_outcome_map(paths: &[String]) -> Result<BTreeMap<String, OutcomeLinkageV3Record>, String> {
    let mut map = BTreeMap::new();
    for path in paths {
        let report = load_outcome_linkage_v3_from_path_or_config(path)?;
        for record in &report.records {
            map.insert(record.row_id.clone(), record.clone());
        }
    }
    Ok(map)
}

fn load_counterfactual_map(
    paths: &[String],
) -> Result<BTreeMap<String, CounterfactualCompletionRecord>, String> {
    let mut map = BTreeMap::new();
    for path in paths {
        let report = load_counterfactual_completion_v2_from_path_or_config(path)?;
        for record in &report.records {
            map.insert(record.row_id.clone(), record.clone());
        }
    }
    Ok(map)
}

fn load_future_window_map(
    requirement_paths: &[String],
    plan_paths: &[String],
) -> Result<BTreeMap<String, bool>, String> {
    let mut map = BTreeMap::new();
    for path in requirement_paths {
        let report = if path.ends_with(".json") {
            super::future_window_requirements::FutureWindowRequirementReport::from_json_path(
                Path::new(path),
            )?
        } else {
            let config =
                super::future_window_requirements::FutureWindowRequirementConfig::from_toml_path(
                    Path::new(path),
                )?;
            super::future_window_requirements::FutureWindowRequirementRunner::default()
                .run(&config)?
        };
        for item in report.items {
            map.insert(
                item.row_id,
                item.missing_future_bars == 0
                    && matches!(
                        item.gap_kind,
                        super::future_window_requirements::FutureWindowGapKind::SufficientFutureBars
                    ),
            );
        }
    }
    for path in plan_paths {
        let plan = load_future_window_scaleout_plan_from_path_or_config(path)?;
        for item in plan.requirement_report.items {
            map.entry(item.row_id).or_insert(
                item.missing_future_bars == 0
                    && matches!(
                        item.gap_kind,
                        super::future_window_requirements::FutureWindowGapKind::SufficientFutureBars
                    ),
            );
        }
    }
    Ok(map)
}

fn determine_item_status(
    inventory_item: &OfficialReadyRowInventoryItem,
    future_window_sufficient: bool,
    outcome_reference_available: bool,
    baseline_reference_available: bool,
    no_trade_counterfactual_available: bool,
    risk_denied_counterfactual_available: bool,
    committee_decision_available: bool,
    risk_decision_available: bool,
    official_complete: bool,
    config: &MultiRowOfficialEvidenceSetConfig,
) -> MultiRowOfficialEvidenceStatus {
    if config.require_no_lookahead_safe && !inventory_item.no_lookahead_safe {
        return MultiRowOfficialEvidenceStatus::NoLookaheadRejected;
    }
    match inventory_item.source_class {
        ComparableEvidenceSourceClass::ControlledDiagnostic => {
            return MultiRowOfficialEvidenceStatus::DiagnosticControlled;
        }
        ComparableEvidenceSourceClass::OfficialCryptoOnly => {
            return MultiRowOfficialEvidenceStatus::CryptoOnly;
        }
        ComparableEvidenceSourceClass::YFinanceResearch => {
            return MultiRowOfficialEvidenceStatus::ResearchOnly;
        }
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => {
            return MultiRowOfficialEvidenceStatus::FixtureOnly;
        }
        ComparableEvidenceSourceClass::Unknown => {
            return MultiRowOfficialEvidenceStatus::SourceIneligible;
        }
        ComparableEvidenceSourceClass::OfficialNonCrypto => {}
    }
    if official_complete {
        return MultiRowOfficialEvidenceStatus::OfficialComplete;
    }
    if config.require_future_window_sufficient && !future_window_sufficient {
        return MultiRowOfficialEvidenceStatus::OfficialPartialMissingFutureWindow;
    }
    if config.require_outcome_reference && !outcome_reference_available {
        return MultiRowOfficialEvidenceStatus::OfficialPartialMissingOutcome;
    }
    if config.require_baseline_reference && !baseline_reference_available {
        return MultiRowOfficialEvidenceStatus::OfficialPartialMissingBaseline;
    }
    if config.require_no_trade_counterfactual && !no_trade_counterfactual_available {
        return MultiRowOfficialEvidenceStatus::OfficialPartialMissingNoTradeCounterfactual;
    }
    if config.require_risk_denied_counterfactual && !risk_denied_counterfactual_available {
        return MultiRowOfficialEvidenceStatus::OfficialPartialMissingRiskDeniedCounterfactual;
    }
    if !committee_decision_available {
        return MultiRowOfficialEvidenceStatus::OfficialPartialMissingCommitteeDecision;
    }
    if !risk_decision_available {
        return MultiRowOfficialEvidenceStatus::OfficialPartialMissingRiskDecision;
    }
    MultiRowOfficialEvidenceStatus::SourceIneligible
}

fn load_provenance_map(paths: &[String]) -> Result<BTreeMap<String, bool>, String> {
    let mut map = BTreeMap::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let value: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        let key = if let Some(local_path) = value.get("local_path").and_then(Value::as_str) {
            stem_key(local_path)
        } else {
            stem_key(path)
        };
        let readiness_eligible = value
            .get("readiness_eligible")
            .and_then(Value::as_bool)
            .unwrap_or(true)
            && value
                .get("official_provider")
                .and_then(Value::as_bool)
                .unwrap_or(true);
        map.insert(key, readiness_eligible);
    }
    Ok(map)
}

fn load_preflight_map(paths: &[String]) -> Result<BTreeMap<String, bool>, String> {
    let mut map = BTreeMap::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let value: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        let key = if let Some(input_path) = value.get("input_path").and_then(Value::as_str) {
            stem_key(input_path)
        } else {
            stem_key(path)
        };
        let ready = value
            .get("final_status")
            .and_then(Value::as_str)
            .map(|status| status == "ReadyForRealEvidence")
            .unwrap_or(true);
        map.insert(key, ready);
    }
    Ok(map)
}

fn descriptor_key(
    descriptor: &super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
) -> String {
    item_key(
        Some(&descriptor.candle_series_id),
        &descriptor.symbol,
        &descriptor.timeframe,
    )
}

fn item_key(candle_series_id: Option<&str>, symbol: &str, timeframe: &str) -> String {
    candle_series_id.map(stem_key).unwrap_or_else(|| {
        format!(
            "{}_{}",
            symbol.to_ascii_lowercase(),
            timeframe.to_ascii_lowercase()
        )
    })
}

fn stem_key(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .to_ascii_lowercase();
    stem.replace("_provenance", "").replace("_preflight", "")
}

fn default_output_root() -> String {
    "target/soma_multi_row_official_evidence".to_string()
}

fn default_max_rows() -> usize {
    1000
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

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
