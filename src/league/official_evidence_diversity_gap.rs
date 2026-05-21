use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::ProviderMarket;

use super::batch_counterfactual_completion::{
    BatchCounterfactualCompletionReport, load_batch_counterfactual_completion_from_path_or_config,
};
use super::batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Report, load_batch_outcome_linkage_v3_from_path_or_config,
};
use super::committee_outcome_reference::CommitteeTripleBarrierLabel;
use super::comparable_committee_evidence::ComparableEvidenceSourceClass;
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceItem, MultiRowOfficialEvidenceSet,
    load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, load_pack_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialEvidenceDiversityGapConfig {
    pub diversity_id: String,
    #[serde(default)]
    pub multi_row_official_set_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_scaleout_paths: Vec<String>,
    #[serde(default)]
    pub batch_outcome_linkage_paths: Vec<String>,
    #[serde(default)]
    pub batch_counterfactual_completion_paths: Vec<String>,
    #[serde(default)]
    pub sufficiency_v2_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_pack_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_closure_v2_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_target_min_rows")]
    pub target_min_rows: usize,
    #[serde(default = "default_target_min_official_complete_rows")]
    pub target_min_official_complete_rows: usize,
    #[serde(default = "default_target_min_symbols")]
    pub target_min_symbols: usize,
    #[serde(default = "default_target_min_timeframes")]
    pub target_min_timeframes: usize,
    #[serde(default = "default_target_min_horizons")]
    pub target_min_horizons: usize,
    #[serde(default = "default_target_min_take_profit")]
    pub target_min_take_profit: usize,
    #[serde(default = "default_target_min_stop_loss")]
    pub target_min_stop_loss: usize,
    #[serde(default = "default_target_min_time_expired")]
    pub target_min_time_expired: usize,
    #[serde(default = "default_target_min_no_trade_counterfactuals")]
    pub target_min_no_trade_counterfactuals: usize,
    #[serde(default = "default_target_min_risk_denied_counterfactuals")]
    pub target_min_risk_denied_counterfactuals: usize,
    #[serde(default = "default_max_single_symbol_concentration_ratio")]
    pub max_single_symbol_concentration_ratio: f64,
    #[serde(default = "default_max_single_outcome_label_ratio")]
    pub max_single_outcome_label_ratio: f64,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceDiversityGapKind {
    InsufficientRows,
    InsufficientOfficialCompleteRows,
    InsufficientSymbolDiversity,
    InsufficientTimeframeDiversity,
    InsufficientHorizonDiversity,
    MissingStopLossOutcomes,
    MissingTimeExpiredOutcomes,
    MissingTakeProfitOutcomes,
    SingleSymbolDominated,
    SingleOutcomeDominated,
    InsufficientNoTradeCounterfactuals,
    InsufficientRiskDeniedCounterfactuals,
    InsufficientBaselineReferences,
    MissingFutureWindows,
    MissingOfficialCandles,
    SourceIneligible,
    DiagnosticOnly,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceDiversityGapStatus {
    NoDiversityGapsDetected,
    NeedMoreOfficialRows,
    NeedMoreSymbols,
    NeedMoreTimeframes,
    NeedMoreHorizons,
    NeedStopLossOutcomes,
    NeedTimeExpiredOutcomes,
    SingleSymbolDominated,
    SingleOutcomeDominated,
    NeedCounterfactualDepth,
    NeedFutureWindowCoverage,
    DiagnosticOnly,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceDiversityGapCell {
    pub market: ProviderMarket,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    #[serde(default)]
    pub horizon_bars: Option<usize>,
    #[serde(default)]
    pub outcome_label: Option<String>,
    pub source_class: ComparableEvidenceSourceClass,
    pub current_count: usize,
    pub target_count: usize,
    pub missing_count: usize,
    #[serde(default)]
    pub impacted_rows: Vec<String>,
    pub buildable_from_existing_data: bool,
    pub buildable_from_local_extension: bool,
    pub buildable_from_provider_collection: bool,
    pub requires_operator_action: bool,
    pub gap_kind: OfficialEvidenceDiversityGapKind,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceDiversityGapMap {
    pub diversity_id: String,
    pub cells: Vec<OfficialEvidenceDiversityGapCell>,
    pub current_total_rows: usize,
    pub current_official_complete_rows: usize,
    pub current_symbols: usize,
    pub current_timeframes: usize,
    pub current_horizons: usize,
    pub current_take_profit: usize,
    pub current_stop_loss: usize,
    pub current_time_expired: usize,
    pub current_no_trade_counterfactuals: usize,
    pub current_risk_denied_counterfactuals: usize,
    pub single_symbol_concentration_ratio: f64,
    pub single_outcome_label_ratio: f64,
    pub buildable_gap_count: usize,
    pub operator_action_gap_count: usize,
    pub gap_status: OfficialEvidenceDiversityGapStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialEvidenceDiversityGapRunner;

impl Default for OfficialEvidenceDiversityGapConfig {
    fn default() -> Self {
        Self {
            diversity_id: "official-evidence-diversity-gap-map".to_string(),
            multi_row_official_set_paths: Vec::new(),
            official_evidence_scaleout_paths: Vec::new(),
            batch_outcome_linkage_paths: Vec::new(),
            batch_counterfactual_completion_paths: Vec::new(),
            sufficiency_v2_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            official_candle_pack_paths: Vec::new(),
            complete_row_closure_v2_paths: Vec::new(),
            output_root: default_output_root(),
            target_min_rows: default_target_min_rows(),
            target_min_official_complete_rows: default_target_min_official_complete_rows(),
            target_min_symbols: default_target_min_symbols(),
            target_min_timeframes: default_target_min_timeframes(),
            target_min_horizons: default_target_min_horizons(),
            target_min_take_profit: default_target_min_take_profit(),
            target_min_stop_loss: default_target_min_stop_loss(),
            target_min_time_expired: default_target_min_time_expired(),
            target_min_no_trade_counterfactuals: default_target_min_no_trade_counterfactuals(),
            target_min_risk_denied_counterfactuals: default_target_min_risk_denied_counterfactuals(
            ),
            max_single_symbol_concentration_ratio: default_max_single_symbol_concentration_ratio(),
            max_single_outcome_label_ratio: default_max_single_outcome_label_ratio(),
            max_bytes: default_max_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialEvidenceDiversityGapConfig {
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
        if self.diversity_id.trim().is_empty() {
            return Err("official evidence diversity gap id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| is_remote_path(path))
        {
            return Err("official evidence diversity gap paths must be local".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official evidence diversity gap max_bytes must be between 1 and 5000000"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.diversity_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.multi_row_official_set_paths
            .iter()
            .chain(self.official_evidence_scaleout_paths.iter())
            .chain(self.batch_outcome_linkage_paths.iter())
            .chain(self.batch_counterfactual_completion_paths.iter())
            .chain(self.sufficiency_v2_paths.iter())
            .chain(self.core_scorecard_paths.iter())
            .chain(self.official_candle_pack_paths.iter())
            .chain(self.complete_row_closure_v2_paths.iter())
            .cloned()
            .collect()
    }
}

impl OfficialEvidenceDiversityGapRunner {
    pub fn run(
        &self,
        config: &OfficialEvidenceDiversityGapConfig,
    ) -> Result<OfficialEvidenceDiversityGapMap, String> {
        config.validate()?;
        let set = load_primary_set(&config.multi_row_official_set_paths)?;
        let outcome_report = load_primary_outcomes(&config.batch_outcome_linkage_paths)?;
        let counterfactual_report =
            load_primary_counterfactuals(&config.batch_counterfactual_completion_paths)?;
        let candle_packs = config
            .official_candle_pack_paths
            .iter()
            .map(|path| load_pack_from_path_or_config(path))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.run_from_inputs(
            config,
            set.as_ref(),
            outcome_report.as_ref(),
            counterfactual_report.as_ref(),
            &candle_packs,
        ))
    }

    pub fn run_from_inputs(
        &self,
        config: &OfficialEvidenceDiversityGapConfig,
        set: Option<&MultiRowOfficialEvidenceSet>,
        outcome_report: Option<&BatchOutcomeLinkageV3Report>,
        counterfactual_report: Option<&BatchCounterfactualCompletionReport>,
        candle_packs: &[OfficialCandleCoveragePack],
    ) -> OfficialEvidenceDiversityGapMap {
        let empty = MultiRowOfficialEvidenceSet {
            set_id: config.diversity_id.clone(),
            items: Vec::new(),
            total_rows: 0,
            official_complete_rows: 0,
            official_partial_rows: 0,
            non_crypto_official_rows: 0,
            crypto_only_rows: 0,
            controlled_rows: 0,
            yfinance_rows: 0,
            fixture_rows: 0,
            outcome_reference_count: 0,
            baseline_reference_count: 0,
            no_trade_counterfactual_count: 0,
            risk_denied_counterfactual_count: 0,
            no_lookahead_safe_count: 0,
            storage_bytes: 0,
            symbol_count: 0,
            timeframe_count: 0,
            horizon_count: 0,
            source_boundaries_preserved: true,
            status:
                super::multi_row_official_evidence::MultiRowOfficialEvidenceStatus::SourceIneligible,
            warnings: Vec::new(),
            reason_codes: Vec::new(),
        };
        let set = set.unwrap_or(&empty);
        let official_items = official_complete_items(set);
        let row_ids = official_items
            .iter()
            .map(|item| item.row_id.clone())
            .collect::<BTreeSet<_>>();

        let current_total_rows = set.total_rows;
        let current_official_complete_rows = official_items.len();
        let current_symbols = official_items
            .iter()
            .map(|item| item.symbol.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let current_timeframes = official_items
            .iter()
            .map(|item| item.timeframe.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let current_horizons = official_items
            .iter()
            .map(|item| item.horizon_bars)
            .collect::<BTreeSet<_>>()
            .len();
        let symbol_counts = official_items
            .iter()
            .fold(BTreeMap::new(), |mut acc, item| {
                *acc.entry(item.symbol.clone()).or_insert(0usize) += 1;
                acc
            });
        let max_symbol_rows = symbol_counts.values().copied().max().unwrap_or_default();
        let single_symbol_concentration_ratio = if current_official_complete_rows == 0 {
            0.0
        } else {
            max_symbol_rows as f64 / current_official_complete_rows as f64
        };

        let label_counts = outcome_label_counts(outcome_report, &row_ids);
        let current_take_profit = *label_counts
            .get(&CommitteeTripleBarrierLabel::TakeProfit)
            .unwrap_or(&0);
        let current_stop_loss = *label_counts
            .get(&CommitteeTripleBarrierLabel::StopLoss)
            .unwrap_or(&0);
        let current_time_expired = *label_counts
            .get(&CommitteeTripleBarrierLabel::TimeExpired)
            .unwrap_or(&0);
        let official_outcomes = current_take_profit + current_stop_loss + current_time_expired;
        let single_outcome_label_ratio = if official_outcomes == 0 {
            0.0
        } else {
            label_counts.values().copied().max().unwrap_or_default() as f64
                / official_outcomes as f64
        };

        let (current_no_trade_counterfactuals, current_risk_denied_counterfactuals) =
            counterfactual_counts(counterfactual_report, &row_ids, set);

        let diagnostics_only = current_official_complete_rows == 0
            && (set.controlled_rows > 0
                || set.crypto_only_rows > 0
                || set.yfinance_rows > 0
                || set.fixture_rows > 0);
        let dominant_symbol = symbol_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(symbol, _)| symbol.clone());
        let dominant_market = official_items
            .iter()
            .find(|item| {
                dominant_symbol
                    .as_ref()
                    .is_some_and(|symbol| symbol == &item.symbol)
            })
            .map(|item| item.market)
            .unwrap_or(ProviderMarket::USEquity);
        let first_market = official_items
            .first()
            .map(|item| item.market)
            .unwrap_or(ProviderMarket::USEquity);
        let buildable_from_existing_data = !candle_packs.is_empty()
            || !config.complete_row_closure_v2_paths.is_empty()
            || !config.core_scorecard_paths.is_empty();
        let buildable_from_provider_collection = true;
        let buildable_from_local_extension = !config.official_candle_pack_paths.is_empty();
        let impacted_rows = official_items
            .iter()
            .map(|item| item.row_id.clone())
            .collect::<Vec<_>>();

        let mut cells = Vec::new();
        let mut push_gap =
            |gap_kind: OfficialEvidenceDiversityGapKind,
             market: ProviderMarket,
             symbol: Option<String>,
             timeframe: Option<String>,
             horizon_bars: Option<usize>,
             outcome_label: Option<String>,
             current_count: usize,
             target_count: usize,
             impacted_rows: Vec<String>,
             buildable_from_existing_data: bool,
             buildable_from_local_extension: bool,
             buildable_from_provider_collection: bool,
             source_class: ComparableEvidenceSourceClass| {
                let missing_count = target_count.saturating_sub(current_count);
                let requires_operator_action = !(buildable_from_existing_data
                    || buildable_from_local_extension
                    || buildable_from_provider_collection);
                cells.push(OfficialEvidenceDiversityGapCell {
                    market,
                    symbol,
                    timeframe,
                    horizon_bars,
                    outcome_label,
                    source_class,
                    current_count,
                    target_count,
                    missing_count,
                    impacted_rows,
                    buildable_from_existing_data,
                    buildable_from_local_extension,
                    buildable_from_provider_collection,
                    requires_operator_action,
                    gap_kind,
                    reason_codes: stable_reason_codes(&[
                        ReasonCode::DeterministicPath,
                        ReasonCode::LocalFileOnly,
                    ]),
                });
            };

        if current_total_rows < config.target_min_rows {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientRows,
                first_market,
                None,
                None,
                None,
                None,
                current_total_rows,
                config.target_min_rows,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_official_complete_rows < config.target_min_official_complete_rows {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientOfficialCompleteRows,
                first_market,
                None,
                None,
                None,
                None,
                current_official_complete_rows,
                config.target_min_official_complete_rows,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_symbols < config.target_min_symbols {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientSymbolDiversity,
                dominant_market,
                dominant_symbol.clone(),
                None,
                None,
                None,
                current_symbols,
                config.target_min_symbols,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_timeframes < config.target_min_timeframes {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientTimeframeDiversity,
                first_market,
                None,
                official_items.first().map(|item| item.timeframe.clone()),
                None,
                None,
                current_timeframes,
                config.target_min_timeframes,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_horizons < config.target_min_horizons {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientHorizonDiversity,
                first_market,
                None,
                None,
                official_items.first().map(|item| item.horizon_bars),
                None,
                current_horizons,
                config.target_min_horizons,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_take_profit < config.target_min_take_profit {
            push_gap(
                OfficialEvidenceDiversityGapKind::MissingTakeProfitOutcomes,
                first_market,
                None,
                None,
                None,
                Some("TakeProfit".to_string()),
                current_take_profit,
                config.target_min_take_profit,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_stop_loss < config.target_min_stop_loss {
            push_gap(
                OfficialEvidenceDiversityGapKind::MissingStopLossOutcomes,
                first_market,
                None,
                None,
                None,
                Some("StopLoss".to_string()),
                current_stop_loss,
                config.target_min_stop_loss,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_time_expired < config.target_min_time_expired {
            push_gap(
                OfficialEvidenceDiversityGapKind::MissingTimeExpiredOutcomes,
                first_market,
                None,
                None,
                None,
                Some("TimeExpired".to_string()),
                current_time_expired,
                config.target_min_time_expired,
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_no_trade_counterfactuals < config.target_min_no_trade_counterfactuals {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientNoTradeCounterfactuals,
                first_market,
                None,
                None,
                None,
                Some("NoTradeCounterfactual".to_string()),
                current_no_trade_counterfactuals,
                config.target_min_no_trade_counterfactuals,
                impacted_rows.clone(),
                true,
                false,
                false,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if current_risk_denied_counterfactuals < config.target_min_risk_denied_counterfactuals {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientRiskDeniedCounterfactuals,
                first_market,
                None,
                None,
                None,
                Some("RiskDeniedCounterfactual".to_string()),
                current_risk_denied_counterfactuals,
                config.target_min_risk_denied_counterfactuals,
                impacted_rows.clone(),
                true,
                false,
                false,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if set.baseline_reference_count < current_official_complete_rows.max(1) {
            push_gap(
                OfficialEvidenceDiversityGapKind::InsufficientBaselineReferences,
                first_market,
                None,
                None,
                None,
                None,
                set.baseline_reference_count,
                current_official_complete_rows.max(1),
                impacted_rows.clone(),
                true,
                false,
                false,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if official_items
            .iter()
            .any(|item| !item.future_window_sufficient)
        {
            let missing_rows = official_items
                .iter()
                .filter(|item| !item.future_window_sufficient)
                .map(|item| item.row_id.clone())
                .collect::<Vec<_>>();
            push_gap(
                OfficialEvidenceDiversityGapKind::MissingFutureWindows,
                first_market,
                None,
                None,
                None,
                None,
                current_official_complete_rows.saturating_sub(missing_rows.len()),
                current_official_complete_rows,
                missing_rows,
                buildable_from_existing_data,
                true,
                true,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if official_items
            .iter()
            .any(|item| !has_matching_pack(item, candle_packs))
        {
            let missing_rows = official_items
                .iter()
                .filter(|item| !has_matching_pack(item, candle_packs))
                .map(|item| item.row_id.clone())
                .collect::<Vec<_>>();
            push_gap(
                OfficialEvidenceDiversityGapKind::MissingOfficialCandles,
                first_market,
                None,
                None,
                None,
                None,
                current_official_complete_rows.saturating_sub(missing_rows.len()),
                current_official_complete_rows,
                missing_rows,
                false,
                true,
                true,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if single_symbol_concentration_ratio > config.max_single_symbol_concentration_ratio {
            push_gap(
                OfficialEvidenceDiversityGapKind::SingleSymbolDominated,
                dominant_market,
                dominant_symbol,
                None,
                None,
                None,
                max_symbol_rows,
                ((config.max_single_symbol_concentration_ratio
                    * current_official_complete_rows as f64)
                    .floor() as usize)
                    .max(1),
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if single_outcome_label_ratio > config.max_single_outcome_label_ratio {
            let dominant_label = label_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(label, _)| format!("{label:?}"));
            let dominant_count = label_counts.values().copied().max().unwrap_or_default();
            push_gap(
                OfficialEvidenceDiversityGapKind::SingleOutcomeDominated,
                first_market,
                None,
                None,
                None,
                dominant_label,
                dominant_count,
                ((config.max_single_outcome_label_ratio * official_outcomes as f64).floor()
                    as usize)
                    .max(1),
                impacted_rows.clone(),
                buildable_from_existing_data,
                buildable_from_local_extension,
                buildable_from_provider_collection,
                ComparableEvidenceSourceClass::OfficialNonCrypto,
            );
        }
        if diagnostics_only {
            push_gap(
                OfficialEvidenceDiversityGapKind::DiagnosticOnly,
                first_market,
                None,
                None,
                None,
                None,
                0,
                1,
                set.items.iter().map(|item| item.row_id.clone()).collect(),
                false,
                false,
                false,
                if set.crypto_only_rows > 0 {
                    ComparableEvidenceSourceClass::OfficialCryptoOnly
                } else if set.controlled_rows > 0 {
                    ComparableEvidenceSourceClass::ControlledDiagnostic
                } else if set.yfinance_rows > 0 {
                    ComparableEvidenceSourceClass::YFinanceResearch
                } else {
                    ComparableEvidenceSourceClass::FixtureArchitectureTest
                },
            );
        }
        if set.total_rows > 0 && current_official_complete_rows == 0 && !diagnostics_only {
            push_gap(
                OfficialEvidenceDiversityGapKind::SourceIneligible,
                first_market,
                None,
                None,
                None,
                None,
                0,
                1,
                set.items.iter().map(|item| item.row_id.clone()).collect(),
                false,
                false,
                false,
                ComparableEvidenceSourceClass::Unknown,
            );
        }

        cells.sort_by(|left, right| {
            left.gap_kind
                .cmp(&right.gap_kind)
                .then(left.market.cmp(&right.market))
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timeframe.cmp(&right.timeframe))
                .then(left.horizon_bars.cmp(&right.horizon_bars))
                .then(left.outcome_label.cmp(&right.outcome_label))
        });

        let buildable_gap_count = cells
            .iter()
            .filter(|cell| {
                cell.buildable_from_existing_data
                    || cell.buildable_from_local_extension
                    || cell.buildable_from_provider_collection
            })
            .count();
        let operator_action_gap_count = cells
            .iter()
            .filter(|cell| cell.requires_operator_action)
            .count();
        let gap_status = determine_gap_status(config, &cells, diagnostics_only);
        let warnings = build_warnings(
            current_official_complete_rows,
            official_outcomes,
            diagnostics_only,
            gap_status,
        );

        OfficialEvidenceDiversityGapMap {
            diversity_id: config.diversity_id.clone(),
            cells,
            current_total_rows,
            current_official_complete_rows,
            current_symbols,
            current_timeframes,
            current_horizons,
            current_take_profit,
            current_stop_loss,
            current_time_expired,
            current_no_trade_counterfactuals,
            current_risk_denied_counterfactuals,
            single_symbol_concentration_ratio,
            single_outcome_label_ratio,
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
                        ReasonCode::DeterministicPath,
                        ReasonCode::LocalFileOnly,
                        ReasonCode::OfficialEvidenceCounted,
                    ])
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl OfficialEvidenceDiversityGapMap {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.diversity_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("diversity_id={}", self.diversity_id),
            format!("current_total_rows={}", self.current_total_rows),
            format!(
                "current_official_complete_rows={}",
                self.current_official_complete_rows
            ),
            format!("current_symbols={}", self.current_symbols),
            format!("current_timeframes={}", self.current_timeframes),
            format!("current_horizons={}", self.current_horizons),
            format!("current_take_profit={}", self.current_take_profit),
            format!("current_stop_loss={}", self.current_stop_loss),
            format!("current_time_expired={}", self.current_time_expired),
            format!(
                "current_no_trade_counterfactuals={}",
                self.current_no_trade_counterfactuals
            ),
            format!(
                "current_risk_denied_counterfactuals={}",
                self.current_risk_denied_counterfactuals
            ),
            format!(
                "single_symbol_concentration_ratio={}",
                self.single_symbol_concentration_ratio
            ),
            format!(
                "single_outcome_label_ratio={}",
                self.single_outcome_label_ratio
            ),
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
                "gap_kind={:?};market={:?};symbol={};timeframe={};horizon_bars={};outcome_label={};current_count={};target_count={};missing_count={};requires_operator_action={};impacted_rows={}",
                cell.gap_kind,
                cell.market,
                cell.symbol.clone().unwrap_or_default(),
                cell.timeframe.clone().unwrap_or_default(),
                cell.horizon_bars.map(|value| value.to_string()).unwrap_or_default(),
                cell.outcome_label.clone().unwrap_or_default(),
                cell.current_count,
                cell.target_count,
                cell.missing_count,
                cell.requires_operator_action,
                cell.impacted_rows.join("|"),
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
            output_dir.join("official_evidence_diversity_gap_map.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_evidence_diversity_gap_map.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_official_evidence_diversity_gap_map_from_path_or_config(
    path: &str,
) -> Result<OfficialEvidenceDiversityGapMap, String> {
    if path.ends_with(".json") {
        OfficialEvidenceDiversityGapMap::from_json_path(Path::new(path))
    } else {
        OfficialEvidenceDiversityGapConfig::from_toml_path(Path::new(path))
            .and_then(|config| OfficialEvidenceDiversityGapRunner::default().run(&config))
    }
}

fn official_complete_items(
    set: &MultiRowOfficialEvidenceSet,
) -> Vec<&MultiRowOfficialEvidenceItem> {
    set.items
        .iter()
        .filter(|item| {
            item.official_complete
                && item.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto
        })
        .collect()
}

fn outcome_label_counts(
    outcome_report: Option<&BatchOutcomeLinkageV3Report>,
    row_ids: &BTreeSet<String>,
) -> BTreeMap<CommitteeTripleBarrierLabel, usize> {
    let mut counts = BTreeMap::new();
    if let Some(report) = outcome_report {
        for record in &report.records {
            if !row_ids.contains(&record.row_id) {
                continue;
            }
            if let Some(reference) = record.outcome_reference.as_ref() {
                if matches!(
                    reference.triple_barrier_label,
                    CommitteeTripleBarrierLabel::TakeProfit
                        | CommitteeTripleBarrierLabel::StopLoss
                        | CommitteeTripleBarrierLabel::TimeExpired
                ) {
                    *counts
                        .entry(reference.triple_barrier_label)
                        .or_insert(0usize) += 1;
                }
            }
        }
    }
    counts
}

fn counterfactual_counts(
    report: Option<&BatchCounterfactualCompletionReport>,
    row_ids: &BTreeSet<String>,
    set: &MultiRowOfficialEvidenceSet,
) -> (usize, usize) {
    if let Some(report) = report {
        let no_trade = report
            .records
            .iter()
            .filter(|record| {
                row_ids.contains(&record.row_id) && record.no_trade_counterfactual_built
            })
            .count();
        let risk_denied = report
            .records
            .iter()
            .filter(|record| {
                row_ids.contains(&record.row_id) && record.risk_denied_counterfactual_built
            })
            .count();
        (no_trade, risk_denied)
    } else {
        (
            set.no_trade_counterfactual_count,
            set.risk_denied_counterfactual_count,
        )
    }
}

fn determine_gap_status(
    config: &OfficialEvidenceDiversityGapConfig,
    cells: &[OfficialEvidenceDiversityGapCell],
    diagnostics_only: bool,
) -> OfficialEvidenceDiversityGapStatus {
    if diagnostics_only {
        return OfficialEvidenceDiversityGapStatus::DiagnosticOnly;
    }
    if cells.iter().any(|cell| {
        cell.gap_kind == OfficialEvidenceDiversityGapKind::SingleOutcomeDominated
            && cell.current_count > 0
            && cell.current_count as f64 > (cell.target_count as f64).max(1.0)
    }) {
        return OfficialEvidenceDiversityGapStatus::SingleOutcomeDominated;
    }
    if cells
        .iter()
        .any(|cell| cell.gap_kind == OfficialEvidenceDiversityGapKind::SingleSymbolDominated)
    {
        return OfficialEvidenceDiversityGapStatus::SingleSymbolDominated;
    }
    if cells
        .iter()
        .any(|cell| cell.gap_kind == OfficialEvidenceDiversityGapKind::MissingTimeExpiredOutcomes)
    {
        return OfficialEvidenceDiversityGapStatus::NeedTimeExpiredOutcomes;
    }
    if cells
        .iter()
        .any(|cell| cell.gap_kind == OfficialEvidenceDiversityGapKind::MissingStopLossOutcomes)
    {
        return OfficialEvidenceDiversityGapStatus::NeedStopLossOutcomes;
    }
    if cells.iter().any(|cell| {
        cell.gap_kind == OfficialEvidenceDiversityGapKind::InsufficientOfficialCompleteRows
    }) {
        return OfficialEvidenceDiversityGapStatus::NeedMoreOfficialRows;
    }
    if cells
        .iter()
        .any(|cell| cell.gap_kind == OfficialEvidenceDiversityGapKind::InsufficientSymbolDiversity)
    {
        return OfficialEvidenceDiversityGapStatus::NeedMoreSymbols;
    }
    if cells.iter().any(|cell| {
        cell.gap_kind == OfficialEvidenceDiversityGapKind::InsufficientTimeframeDiversity
    }) {
        return OfficialEvidenceDiversityGapStatus::NeedMoreTimeframes;
    }
    if cells
        .iter()
        .any(|cell| cell.gap_kind == OfficialEvidenceDiversityGapKind::InsufficientHorizonDiversity)
    {
        return OfficialEvidenceDiversityGapStatus::NeedMoreHorizons;
    }
    if cells.iter().any(|cell| {
        matches!(
            cell.gap_kind,
            OfficialEvidenceDiversityGapKind::InsufficientNoTradeCounterfactuals
                | OfficialEvidenceDiversityGapKind::InsufficientRiskDeniedCounterfactuals
        )
    }) {
        return OfficialEvidenceDiversityGapStatus::NeedCounterfactualDepth;
    }
    if cells.iter().any(|cell| {
        matches!(
            cell.gap_kind,
            OfficialEvidenceDiversityGapKind::MissingFutureWindows
                | OfficialEvidenceDiversityGapKind::MissingOfficialCandles
        )
    }) {
        return OfficialEvidenceDiversityGapStatus::NeedFutureWindowCoverage;
    }
    if config.target_min_rows == 0 || cells.is_empty() {
        return OfficialEvidenceDiversityGapStatus::NoDiversityGapsDetected;
    }
    OfficialEvidenceDiversityGapStatus::NeedMoreEvidence
}

fn build_warnings(
    official_complete_rows: usize,
    official_outcomes: usize,
    diagnostics_only: bool,
    gap_status: OfficialEvidenceDiversityGapStatus,
) -> Vec<String> {
    let mut warnings = vec![
        "diversity gap mapping is research-only and never implies profitability or live readiness"
            .to_string(),
    ];
    if official_complete_rows <= 2 {
        warnings.push(
            "one or two official complete rows remain too small for committee research sufficiency"
                .to_string(),
        );
    }
    if official_outcomes > 0 {
        warnings.push(
            "mixed outcome labels can improve diversity status but do not imply profitable edge"
                .to_string(),
        );
    }
    if diagnostics_only {
        warnings.push(
            "diagnostic, crypto-only, yfinance, or fixture rows remain ineligible for official sufficiency"
                .to_string(),
        );
    }
    if matches!(
        gap_status,
        OfficialEvidenceDiversityGapStatus::SingleOutcomeDominated
    ) {
        warnings.push(
            "single-outcome dominance remains a conservative block on research readiness"
                .to_string(),
        );
    }
    warnings
}

fn has_matching_pack(
    item: &MultiRowOfficialEvidenceItem,
    packs: &[OfficialCandleCoveragePack],
) -> bool {
    packs.iter().any(|pack| {
        pack.descriptors.iter().any(|descriptor| {
            descriptor.official_readiness_eligible
                && descriptor.symbol.eq_ignore_ascii_case(&item.symbol)
                && descriptor.timeframe.eq_ignore_ascii_case(&item.timeframe)
                && descriptor.market == item.market
        })
    })
}

fn load_primary_set(paths: &[String]) -> Result<Option<MultiRowOfficialEvidenceSet>, String> {
    paths
        .first()
        .map(|path| load_multi_row_official_evidence_set_from_path_or_config(path))
        .transpose()
}

fn load_primary_outcomes(paths: &[String]) -> Result<Option<BatchOutcomeLinkageV3Report>, String> {
    paths
        .first()
        .map(|path| load_batch_outcome_linkage_v3_from_path_or_config(path))
        .transpose()
}

fn load_primary_counterfactuals(
    paths: &[String],
) -> Result<Option<BatchCounterfactualCompletionReport>, String> {
    paths
        .first()
        .map(|path| load_batch_counterfactual_completion_from_path_or_config(path))
        .transpose()
}

fn default_output_root() -> String {
    "target/soma_official_evidence_diversity_gap".to_string()
}

fn default_target_min_rows() -> usize {
    3
}

fn default_target_min_official_complete_rows() -> usize {
    3
}

fn default_target_min_symbols() -> usize {
    2
}

fn default_target_min_timeframes() -> usize {
    2
}

fn default_target_min_horizons() -> usize {
    2
}

fn default_target_min_take_profit() -> usize {
    1
}

fn default_target_min_stop_loss() -> usize {
    1
}

fn default_target_min_time_expired() -> usize {
    1
}

fn default_target_min_no_trade_counterfactuals() -> usize {
    2
}

fn default_target_min_risk_denied_counterfactuals() -> usize {
    2
}

fn default_max_single_symbol_concentration_ratio() -> f64 {
    0.8
}

fn default_max_single_outcome_label_ratio() -> f64 {
    0.8
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn is_remote_path(value: &str) -> bool {
    value.contains("://")
}
