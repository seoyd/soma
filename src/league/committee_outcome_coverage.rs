use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::committee_counterfactual_builder::{
    CommitteeCounterfactualRecord, CommitteeCounterfactualType, fixture_source_kind,
    horizon_bars_for_row, normalize_symbol,
};
use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioPack;
use super::committee_scenario_loader::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcomeCoverageConfig {
    pub coverage_id: String,
    #[serde(default)]
    pub official_benchmark_report_paths: Vec<String>,
    #[serde(default)]
    pub outcome_linked_pack_paths: Vec<String>,
    #[serde(default)]
    pub scenario_pack_paths: Vec<String>,
    #[serde(default)]
    pub committee_benchmark_bundle_paths: Vec<String>,
    #[serde(default)]
    pub candle_series_paths: Vec<String>,
    #[serde(default)]
    pub baseline_reference_paths: Vec<String>,
    #[serde(default)]
    pub external_reference_paths: Vec<String>,
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_official_rows: bool,
    #[serde(default)]
    pub allow_crypto_only: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub allow_estimated_counterfactuals: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeOutcomeCoverageStatus {
    HealthyCoverage,
    NeedMoreOfficialRows,
    NeedMoreOutcomeLinks,
    NeedMoreBaselineReferences,
    NeedMoreNoTradeCounterfactuals,
    NeedMoreRiskDeniedCounterfactuals,
    CryptoOnlyCoverage,
    ResearchOnlyCoverage,
    FixtureOnlyCoverage,
    InsufficientCoverage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeCoverageCell {
    pub source_kind: String,
    pub market: ProviderMarket,
    pub symbol: String,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub row_count: usize,
    pub outcome_linked_count: usize,
    pub baseline_linked_count: usize,
    pub external_linked_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub official_row_count: usize,
    pub research_only_row_count: usize,
    pub fixture_row_count: usize,
    pub crypto_only_row_count: usize,
    pub no_lookahead_safe_count: usize,
    pub missing_outcome_count: usize,
    pub missing_baseline_count: usize,
    pub missing_counterfactual_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcomeCoverageReport {
    pub coverage_id: String,
    pub cells: Vec<OutcomeCoverageCell>,
    pub total_rows: usize,
    pub official_rows: usize,
    pub row_level_rows: usize,
    pub summary_derived_rows: usize,
    pub outcome_linked_rows: usize,
    pub baseline_linked_rows: usize,
    pub external_linked_rows: usize,
    pub no_trade_counterfactuals: usize,
    pub risk_denied_counterfactuals: usize,
    pub no_lookahead_violations: usize,
    pub source_summary: String,
    pub coverage_status: CommitteeOutcomeCoverageStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageCellKey {
    source_kind: String,
    market: ProviderMarket,
    symbol: String,
    timeframe: String,
    horizon_bars: usize,
}

impl Default for CommitteeOutcomeCoverageConfig {
    fn default() -> Self {
        Self {
            coverage_id: "committee_outcome_coverage".to_string(),
            official_benchmark_report_paths: Vec::new(),
            outcome_linked_pack_paths: Vec::new(),
            scenario_pack_paths: Vec::new(),
            committee_benchmark_bundle_paths: Vec::new(),
            candle_series_paths: Vec::new(),
            baseline_reference_paths: Vec::new(),
            external_reference_paths: Vec::new(),
            output_root: "target/soma_committee_outcome_coverage".to_string(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_official_rows: true,
            allow_crypto_only: false,
            allow_yfinance_research: false,
            allow_fixture: false,
            allow_estimated_counterfactuals: false,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::CommitteeOutcomeCoverageBuilt],
        }
    }
}

impl CommitteeOutcomeCoverageConfig {
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
        let paths = self
            .official_benchmark_report_paths
            .iter()
            .chain(self.outcome_linked_pack_paths.iter())
            .chain(self.scenario_pack_paths.iter())
            .chain(self.committee_benchmark_bundle_paths.iter())
            .chain(self.candle_series_paths.iter())
            .chain(self.baseline_reference_paths.iter())
            .chain(self.external_reference_paths.iter())
            .chain(std::iter::once(&self.output_root));
        if paths.clone().any(|path| path.contains("://")) {
            return Err("committee outcome coverage paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "committee outcome coverage max_rows must be between 1 and 100".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > 10 {
            return Err(
                "committee outcome coverage max_symbols must be between 1 and 10".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "committee outcome coverage max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.coverage_id)
    }
}

pub fn build_committee_outcome_coverage_report(
    config: &CommitteeOutcomeCoverageConfig,
    packs: &[super::official_committee_pack::OfficialCommitteeScenarioPack],
    linked_packs: &[OutcomeLinkedCommitteeScenarioPack],
    counterfactual_records: &[CommitteeCounterfactualRecord],
) -> CommitteeOutcomeCoverageReport {
    let linked_by_row_id = linked_packs
        .iter()
        .flat_map(|pack| {
            pack.linked_rows
                .iter()
                .map(|row| (row.scenario_row.scenario_row_id.clone(), row))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeMap<_, _>>();
    let counterfactuals_by_row_id = counterfactual_records.iter().fold(
        BTreeMap::<String, Vec<&CommitteeCounterfactualRecord>>::new(),
        |mut acc, record| {
            acc.entry(record.scenario_row_id.clone())
                .or_default()
                .push(record);
            acc
        },
    );
    let mut cells = BTreeMap::<CoverageCellKey, OutcomeCoverageCell>::new();
    let mut total_rows = 0usize;
    let mut official_rows = 0usize;
    let mut row_level_rows = 0usize;
    let mut summary_derived_rows = 0usize;
    let mut source_counts = BTreeMap::<String, usize>::new();

    for pack in packs {
        for row in &pack.rows {
            total_rows += 1;
            if row.evidence_source_kind.readiness_eligible() {
                official_rows += 1;
            }
            if row.materialization_level == CommitteeScenarioMaterializationLevel::RowLevel {
                row_level_rows += 1;
            } else {
                summary_derived_rows += 1;
            }
            let key = CoverageCellKey {
                source_kind: format!("{:?}", row.evidence_source_kind),
                market: row.market,
                symbol: normalize_symbol(&row.symbol),
                timeframe: timeframe_for_row(row),
                horizon_bars: horizon_bars_for_row(row, 24),
            };
            *source_counts.entry(key.source_kind.clone()).or_insert(0) += 1;
            let linked = linked_by_row_id.get(&row.scenario_row_id).copied();
            let counterfactuals = counterfactuals_by_row_id
                .get(&row.scenario_row_id)
                .cloned()
                .unwrap_or_default();
            let mut reason_codes = row.reason_codes.clone();
            if let Some(linked_row) = linked {
                reason_codes.extend(linked_row.reason_codes.clone());
                if let Some(reference) = &linked_row.outcome_reference {
                    reason_codes.extend(reference.reason_codes.clone());
                }
            }
            for record in &counterfactuals {
                reason_codes.extend(record.reason_codes.clone());
            }
            let entry = cells.entry(key).or_insert_with(|| OutcomeCoverageCell {
                source_kind: format!("{:?}", row.evidence_source_kind),
                market: row.market,
                symbol: normalize_symbol(&row.symbol),
                timeframe: timeframe_for_row(row),
                horizon_bars: horizon_bars_for_row(row, 24),
                row_count: 0,
                outcome_linked_count: 0,
                baseline_linked_count: 0,
                external_linked_count: 0,
                no_trade_counterfactual_count: 0,
                risk_denied_counterfactual_count: 0,
                official_row_count: 0,
                research_only_row_count: 0,
                fixture_row_count: 0,
                crypto_only_row_count: 0,
                no_lookahead_safe_count: 0,
                missing_outcome_count: 0,
                missing_baseline_count: 0,
                missing_counterfactual_count: 0,
                reason_codes: Vec::new(),
            });
            entry.row_count += 1;
            if row.evidence_source_kind.readiness_eligible() {
                entry.official_row_count += 1;
            }
            if row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch {
                entry.research_only_row_count += 1;
            }
            if fixture_source_kind(row) {
                entry.fixture_row_count += 1;
            }
            if row.market == ProviderMarket::Crypto {
                entry.crypto_only_row_count += 1;
            }

            let no_trade_built = counterfactuals.iter().any(|record| {
                record.counterfactual_type == CommitteeCounterfactualType::NoTrade
                    && (record.built()
                        && (config.allow_estimated_counterfactuals || !record.diagnostic_only))
            });
            let risk_denied_built = counterfactuals.iter().any(|record| {
                record.counterfactual_type == CommitteeCounterfactualType::RiskDenied
                    && (record.built()
                        && (config.allow_estimated_counterfactuals || !record.diagnostic_only))
            });
            if no_trade_built {
                entry.no_trade_counterfactual_count += 1;
            }
            if risk_denied_built {
                entry.risk_denied_counterfactual_count += 1;
            }
            if !no_trade_built {
                entry.missing_counterfactual_count += 1;
            }
            if !risk_denied_built {
                entry.missing_counterfactual_count += 1;
            }

            let no_lookahead_safe = linked
                .and_then(|linked_row| linked_row.outcome_reference.as_ref())
                .map(|reference| reference.no_lookahead_safe)
                .unwrap_or(true)
                && counterfactuals.iter().all(|record| {
                    !record.built()
                        || config.allow_estimated_counterfactuals
                        || !record.diagnostic_only
                })
                && counterfactuals
                    .iter()
                    .all(|record| record.no_lookahead_safe || !record.built());
            if no_lookahead_safe {
                entry.no_lookahead_safe_count += 1;
            }

            if let Some(linked_row) = linked {
                if linked_row.outcome_reference.is_some() {
                    entry.outcome_linked_count += 1;
                } else {
                    entry.missing_outcome_count += 1;
                }
                if linked_row.baseline_reference.is_some() {
                    entry.baseline_linked_count += 1;
                } else {
                    entry.missing_baseline_count += 1;
                }
                if linked_row.external_reference.is_some() {
                    entry.external_linked_count += 1;
                }
            } else {
                entry.missing_outcome_count += 1;
                entry.missing_baseline_count += 1;
            }
            entry.reason_codes = stable_reason_codes(
                &entry
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain(reason_codes)
                    .collect::<Vec<_>>(),
            );
        }
    }

    let cells = cells.into_values().collect::<Vec<_>>();
    let outcome_linked_rows = cells
        .iter()
        .map(|cell| cell.outcome_linked_count)
        .sum::<usize>();
    let baseline_linked_rows = cells
        .iter()
        .map(|cell| cell.baseline_linked_count)
        .sum::<usize>();
    let external_linked_rows = cells
        .iter()
        .map(|cell| cell.external_linked_count)
        .sum::<usize>();
    let no_trade_counterfactuals = cells
        .iter()
        .map(|cell| cell.no_trade_counterfactual_count)
        .sum::<usize>();
    let risk_denied_counterfactuals = cells
        .iter()
        .map(|cell| cell.risk_denied_counterfactual_count)
        .sum::<usize>();
    let no_lookahead_violations = total_rows.saturating_sub(
        cells
            .iter()
            .map(|cell| cell.no_lookahead_safe_count)
            .sum::<usize>(),
    );
    let source_summary = source_counts
        .into_iter()
        .map(|(source_kind, count)| format!("{source_kind}={count}"))
        .collect::<Vec<_>>()
        .join("|");
    let research_only_rows = cells
        .iter()
        .map(|cell| cell.research_only_row_count)
        .sum::<usize>();
    let fixture_rows = cells
        .iter()
        .map(|cell| cell.fixture_row_count)
        .sum::<usize>();
    let crypto_rows = cells
        .iter()
        .map(|cell| cell.crypto_only_row_count)
        .sum::<usize>();
    let coverage_status = if total_rows == 0 {
        CommitteeOutcomeCoverageStatus::InsufficientCoverage
    } else if fixture_rows == total_rows {
        CommitteeOutcomeCoverageStatus::FixtureOnlyCoverage
    } else if research_only_rows == total_rows {
        CommitteeOutcomeCoverageStatus::ResearchOnlyCoverage
    } else if crypto_rows == total_rows {
        CommitteeOutcomeCoverageStatus::CryptoOnlyCoverage
    } else if config.require_no_lookahead_safe && no_lookahead_violations > 0 {
        CommitteeOutcomeCoverageStatus::InsufficientCoverage
    } else if config.require_official_rows && official_rows == 0 {
        CommitteeOutcomeCoverageStatus::NeedMoreOfficialRows
    } else if outcome_linked_rows == 0 {
        CommitteeOutcomeCoverageStatus::NeedMoreOutcomeLinks
    } else if baseline_linked_rows == 0 {
        CommitteeOutcomeCoverageStatus::NeedMoreBaselineReferences
    } else if no_trade_counterfactuals == 0 {
        CommitteeOutcomeCoverageStatus::NeedMoreNoTradeCounterfactuals
    } else if risk_denied_counterfactuals == 0 {
        CommitteeOutcomeCoverageStatus::NeedMoreRiskDeniedCounterfactuals
    } else {
        CommitteeOutcomeCoverageStatus::HealthyCoverage
    };
    CommitteeOutcomeCoverageReport {
        coverage_id: config.coverage_id.clone(),
        cells,
        total_rows,
        official_rows,
        row_level_rows,
        summary_derived_rows,
        outcome_linked_rows,
        baseline_linked_rows,
        external_linked_rows,
        no_trade_counterfactuals,
        risk_denied_counterfactuals,
        no_lookahead_violations,
        source_summary,
        coverage_status,
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::CommitteeOutcomeCoverageBuilt])
                .collect::<Vec<_>>(),
        ),
    }
}

impl CommitteeOutcomeCoverageReport {
    pub fn row_level_ratio(&self) -> f64 {
        self.row_level_rows as f64 / self.total_rows.max(1) as f64
    }

    pub fn summary_derived_ratio(&self) -> f64 {
        self.summary_derived_rows as f64 / self.total_rows.max(1) as f64
    }

    pub fn research_only_ratio(&self) -> f64 {
        self.cells
            .iter()
            .map(|cell| cell.research_only_row_count)
            .sum::<usize>() as f64
            / self.total_rows.max(1) as f64
    }

    pub fn fixture_ratio(&self) -> f64 {
        self.cells
            .iter()
            .map(|cell| cell.fixture_row_count)
            .sum::<usize>() as f64
            / self.total_rows.max(1) as f64
    }

    pub fn crypto_only_ratio(&self) -> f64 {
        self.cells
            .iter()
            .map(|cell| cell.crypto_only_row_count)
            .sum::<usize>() as f64
            / self.total_rows.max(1) as f64
    }

    pub fn official_non_crypto_rows(&self) -> usize {
        self.cells
            .iter()
            .filter(|cell| cell.market != ProviderMarket::Crypto)
            .map(|cell| cell.official_row_count)
            .sum()
    }

    pub fn source_diversity_count(&self) -> usize {
        self.cells
            .iter()
            .map(|cell| cell.source_kind.clone())
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("coverage_id={}", self.coverage_id),
            format!("coverage_status={:?}", self.coverage_status),
            format!("total_rows={}", self.total_rows),
            format!("official_rows={}", self.official_rows),
            format!("row_level_rows={}", self.row_level_rows),
            format!("summary_derived_rows={}", self.summary_derived_rows),
            format!("outcome_linked_rows={}", self.outcome_linked_rows),
            format!("baseline_linked_rows={}", self.baseline_linked_rows),
            format!("external_linked_rows={}", self.external_linked_rows),
            format!("no_trade_counterfactuals={}", self.no_trade_counterfactuals),
            format!(
                "risk_denied_counterfactuals={}",
                self.risk_denied_counterfactuals
            ),
            format!("no_lookahead_violations={}", self.no_lookahead_violations),
            format!("source_summary={}", self.source_summary),
        ];
        for cell in &self.cells {
            lines.push(format!(
                "cell=source_kind:{};market:{:?};symbol:{};timeframe:{};horizon_bars:{};row_count:{};outcome_linked_count:{};baseline_linked_count:{};external_linked_count:{};no_trade_counterfactual_count:{};risk_denied_counterfactual_count:{};official_row_count:{};research_only_row_count:{};fixture_row_count:{};crypto_only_row_count:{};no_lookahead_safe_count:{};missing_outcome_count:{};missing_baseline_count:{};missing_counterfactual_count:{}",
                cell.source_kind,
                cell.market,
                cell.symbol,
                cell.timeframe,
                cell.horizon_bars,
                cell.row_count,
                cell.outcome_linked_count,
                cell.baseline_linked_count,
                cell.external_linked_count,
                cell.no_trade_counterfactual_count,
                cell.risk_denied_counterfactual_count,
                cell.official_row_count,
                cell.research_only_row_count,
                cell.fixture_row_count,
                cell.crypto_only_row_count,
                cell.no_lookahead_safe_count,
                cell.missing_outcome_count,
                cell.missing_baseline_count,
                cell.missing_counterfactual_count,
            ));
        }
        lines.join("\n")
    }
}

fn timeframe_for_row(row: &CommitteeScenarioRow) -> String {
    match row.target_horizon {
        super::persona_card_lite::PersonaHorizon::Intraday => "intraday".to_string(),
        super::persona_card_lite::PersonaHorizon::Swing => "swing".to_string(),
        super::persona_card_lite::PersonaHorizon::MultiDay => "multi_day".to_string(),
        super::persona_card_lite::PersonaHorizon::LongTerm => "long_term".to_string(),
    }
}

fn default_max_rows() -> usize {
    100
}

fn default_max_symbols() -> usize {
    3
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
