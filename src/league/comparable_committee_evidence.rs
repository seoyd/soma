use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::committee_scenario_loader::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow, CommitteeScenarioSourceKind,
};
use super::official_row_injection::{OfficialEvidenceBoundary, classify_row_boundary};
use super::persona_card_lite::PersonaHorizon;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparableCommitteeEvidenceConfig {
    pub comparable_id: String,
    #[serde(default)]
    pub official_replication_report_paths: Vec<String>,
    #[serde(default)]
    pub official_committee_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub committee_benchmark_bundle_paths: Vec<String>,
    #[serde(default)]
    pub outcome_coverage_bundle_paths: Vec<String>,
    #[serde(default)]
    pub reference_pack_bundle_paths: Vec<String>,
    #[serde(default)]
    pub core_performance_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub source_aware_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub yahoo_research_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_official_for_usefulness_claim: bool,
    #[serde(default = "default_true")]
    pub allow_controlled_evidence: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_yfinance_research: bool,
    #[serde(default = "default_true")]
    pub allow_fixture: bool,
    #[serde(default)]
    pub allow_summary_derived_rows: bool,
    #[serde(default = "default_true")]
    pub require_outcome_reference: bool,
    #[serde(default = "default_true")]
    pub require_baseline_reference: bool,
    #[serde(default = "default_true")]
    pub require_no_trade_counterfactual: bool,
    #[serde(default = "default_true")]
    pub require_risk_denied_counterfactual: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComparableEvidenceSourceClass {
    OfficialNonCrypto,
    OfficialCryptoOnly,
    ControlledDiagnostic,
    YFinanceResearch,
    FixtureArchitectureTest,
    SyntheticTest,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparableCommitteeEvidenceRow {
    pub row_id: String,
    pub symbol: String,
    pub market: ProviderMarket,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub timestamp_ms: u64,
    pub source_kind: String,
    pub source_class: ComparableEvidenceSourceClass,
    #[serde(default)]
    pub scenario_row_id: Option<String>,
    #[serde(default)]
    pub committee_decision_id: Option<String>,
    pub committee_final_action: String,
    #[serde(default)]
    pub chair_decision: Option<String>,
    #[serde(default)]
    pub risk_governor_decision: Option<String>,
    #[serde(default)]
    pub baseline_action: Option<String>,
    #[serde(default)]
    pub external_action: Option<String>,
    pub no_trade_baseline_action: String,
    #[serde(default)]
    pub outcome_label: Option<String>,
    #[serde(default)]
    pub net_return_pct: Option<f64>,
    #[serde(default)]
    pub cost_bps: f64,
    #[serde(default)]
    pub slippage_bps: f64,
    #[serde(default)]
    pub committee_vs_baseline_delta: Option<f64>,
    #[serde(default)]
    pub committee_vs_notrade_delta: Option<f64>,
    #[serde(default)]
    pub risk_denied_value_proxy: Option<f64>,
    #[serde(default)]
    pub no_trade_value_proxy: Option<f64>,
    #[serde(default)]
    pub outcome_reference_available: bool,
    #[serde(default)]
    pub baseline_reference_available: bool,
    #[serde(default)]
    pub no_trade_counterfactual_available: bool,
    #[serde(default)]
    pub risk_denied_counterfactual_available: bool,
    #[serde(default)]
    pub external_reference_available: bool,
    #[serde(default)]
    pub row_level: bool,
    #[serde(default)]
    pub summary_derived: bool,
    #[serde(default)]
    pub no_lookahead_safe: bool,
    #[serde(default)]
    pub official_readiness_eligible: bool,
    #[serde(default)]
    pub diagnostic_only: bool,
    #[serde(default)]
    pub candle_coverage_available: bool,
    #[serde(default)]
    pub matched_candle_series_id: Option<String>,
    #[serde(default)]
    pub candle_match_status: Option<String>,
    #[serde(default)]
    pub candle_official_ready_match: bool,
    #[serde(default)]
    pub candle_benchmark_ready_match: bool,
    #[serde(default)]
    pub candle_diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparableCommitteeEvidenceBundle {
    pub comparable_id: String,
    pub rows: Vec<ComparableCommitteeEvidenceRow>,
    pub complete_rows: usize,
    pub incomplete_rows: usize,
    pub official_rows: usize,
    pub non_crypto_official_rows: usize,
    pub crypto_only_rows: usize,
    pub controlled_rows: usize,
    pub yfinance_rows: usize,
    pub fixture_rows: usize,
    pub row_level_count: usize,
    pub summary_derived_count: usize,
    pub outcome_reference_count: usize,
    pub baseline_reference_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub external_reference_count: usize,
    pub no_lookahead_safe_count: usize,
    pub storage_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ComparableCommitteeEvidenceConfig {
    fn default() -> Self {
        Self {
            comparable_id: "comparable-committee-evidence".to_string(),
            official_replication_report_paths: Vec::new(),
            official_committee_benchmark_paths: Vec::new(),
            committee_benchmark_bundle_paths: Vec::new(),
            outcome_coverage_bundle_paths: Vec::new(),
            reference_pack_bundle_paths: Vec::new(),
            core_performance_scorecard_paths: Vec::new(),
            source_aware_benchmark_paths: Vec::new(),
            yahoo_research_report_paths: Vec::new(),
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_official_for_usefulness_claim: true,
            allow_controlled_evidence: true,
            allow_crypto_only: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            allow_summary_derived_rows: false,
            require_outcome_reference: true,
            require_baseline_reference: true,
            require_no_trade_counterfactual: true,
            require_risk_denied_counterfactual: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ComparableCommitteeEvidenceConfig {
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
        if self.comparable_id.trim().is_empty() {
            return Err("comparable evidence id must not be empty".to_string());
        }
        if self
            .all_artifact_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("comparable evidence paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err("comparable evidence max_rows must be between 1 and 500".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("comparable evidence max_symbols must be between 1 and 5".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err("comparable evidence max_bytes must be between 1 and 5000000".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.comparable_id)
    }

    pub fn all_artifact_paths(&self) -> Vec<String> {
        self.official_replication_report_paths
            .iter()
            .chain(self.official_committee_benchmark_paths.iter())
            .chain(self.committee_benchmark_bundle_paths.iter())
            .chain(self.outcome_coverage_bundle_paths.iter())
            .chain(self.reference_pack_bundle_paths.iter())
            .chain(self.core_performance_scorecard_paths.iter())
            .chain(self.source_aware_benchmark_paths.iter())
            .chain(self.yahoo_research_report_paths.iter())
            .cloned()
            .collect()
    }
}

impl ComparableCommitteeEvidenceRow {
    pub fn from_scenario_row(row: &CommitteeScenarioRow) -> Self {
        let source_class = classify_source_class_for_row(row);
        let row_level =
            row.materialization_level == CommitteeScenarioMaterializationLevel::RowLevel;
        let summary_derived = !row_level || row.reason_codes.contains(&ReasonCode::SummaryDerived);
        let official_readiness_eligible = matches!(
            source_class,
            ComparableEvidenceSourceClass::OfficialNonCrypto
        ) && row_level
            && !summary_derived;
        let diagnostic_only = matches!(
            source_class,
            ComparableEvidenceSourceClass::ControlledDiagnostic
                | ComparableEvidenceSourceClass::YFinanceResearch
                | ComparableEvidenceSourceClass::FixtureArchitectureTest
                | ComparableEvidenceSourceClass::SyntheticTest
        );
        Self {
            row_id: row.scenario_row_id.clone(),
            symbol: row.symbol.clone(),
            market: row.market,
            timeframe: timeframe_for_row(row),
            horizon_bars: horizon_bars_for_row(row),
            timestamp_ms: row.timestamp_ms,
            source_kind: format!("{:?}", row.source_kind),
            source_class,
            scenario_row_id: Some(row.scenario_row_id.clone()),
            committee_decision_id: None,
            committee_final_action: "Unknown".to_string(),
            chair_decision: None,
            risk_governor_decision: None,
            baseline_action: row.baseline_signal_summary.clone(),
            external_action: row.external_prediction_summary.clone(),
            no_trade_baseline_action: "NoTrade".to_string(),
            outcome_label: None,
            net_return_pct: None,
            cost_bps: 0.0,
            slippage_bps: 0.0,
            committee_vs_baseline_delta: None,
            committee_vs_notrade_delta: None,
            risk_denied_value_proxy: None,
            no_trade_value_proxy: None,
            outcome_reference_available: row.outcome_reference.is_some(),
            baseline_reference_available: row.baseline_signal_summary.is_some(),
            no_trade_counterfactual_available: row.no_trade_counterfactual.is_some(),
            risk_denied_counterfactual_available: row.risk_denial_counterfactual.is_some(),
            external_reference_available: row.external_prediction_summary.is_some(),
            row_level,
            summary_derived,
            no_lookahead_safe: !row
                .reason_codes
                .contains(&ReasonCode::RejectedNoLookaheadReference),
            official_readiness_eligible,
            diagnostic_only,
            candle_coverage_available: false,
            matched_candle_series_id: None,
            candle_match_status: None,
            candle_official_ready_match: false,
            candle_benchmark_ready_match: false,
            candle_diagnostic_only: false,
            reason_codes: stable_reason_codes(&row.reason_codes),
        }
    }

    pub fn missing_references(
        &self,
        config: &ComparableCommitteeEvidenceConfig,
    ) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if config.require_outcome_reference && !self.outcome_reference_available {
            missing.push("outcome");
        }
        if config.require_baseline_reference && !self.baseline_reference_available {
            missing.push("baseline");
        }
        if config.require_no_trade_counterfactual && !self.no_trade_counterfactual_available {
            missing.push("no_trade");
        }
        if config.require_risk_denied_counterfactual && !self.risk_denied_counterfactual_available {
            missing.push("risk_denied");
        }
        missing
    }

    pub fn complete(&self, config: &ComparableCommitteeEvidenceConfig) -> bool {
        self.no_lookahead_safe && self.missing_references(config).is_empty()
    }

    pub fn official_complete(&self, config: &ComparableCommitteeEvidenceConfig) -> bool {
        self.complete(config)
            && self.official_readiness_eligible
            && (!config.require_official_for_usefulness_claim
                || self.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
            && (config.allow_summary_derived_rows || !self.summary_derived)
    }

    pub fn completeness_score(&self, config: &ComparableCommitteeEvidenceConfig) -> usize {
        let mut score = 0usize;
        if self.outcome_reference_available {
            score += 1;
        }
        if self.baseline_reference_available {
            score += 1;
        }
        if self.no_trade_counterfactual_available {
            score += 1;
        }
        if self.risk_denied_counterfactual_available {
            score += 1;
        }
        if self.external_reference_available {
            score += 1;
        }
        if self.complete(config) {
            score += 8;
        }
        if self.row_level {
            score += 4;
        }
        if !self.summary_derived {
            score += 2;
        }
        if self.official_readiness_eligible {
            score += 2;
        }
        if !self.diagnostic_only {
            score += 1;
        }
        score
    }
}

impl ComparableCommitteeEvidenceBundle {
    pub fn from_rows(
        config: &ComparableCommitteeEvidenceConfig,
        mut rows: Vec<ComparableCommitteeEvidenceRow>,
    ) -> Self {
        rows.sort_by(|left, right| {
            left.row_id
                .cmp(&right.row_id)
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timestamp_ms.cmp(&right.timestamp_ms))
                .then(left.source_kind.cmp(&right.source_kind))
        });
        let complete_rows = rows.iter().filter(|row| row.complete(config)).count();
        let incomplete_rows = rows.len().saturating_sub(complete_rows);
        let official_rows = rows
            .iter()
            .filter(|row| {
                matches!(
                    row.source_class,
                    ComparableEvidenceSourceClass::OfficialNonCrypto
                        | ComparableEvidenceSourceClass::OfficialCryptoOnly
                )
            })
            .count();
        let non_crypto_official_rows = rows
            .iter()
            .filter(|row| row.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
            .count();
        let crypto_only_rows = rows
            .iter()
            .filter(|row| row.source_class == ComparableEvidenceSourceClass::OfficialCryptoOnly)
            .count();
        let controlled_rows = rows
            .iter()
            .filter(|row| row.source_class == ComparableEvidenceSourceClass::ControlledDiagnostic)
            .count();
        let yfinance_rows = rows
            .iter()
            .filter(|row| row.source_class == ComparableEvidenceSourceClass::YFinanceResearch)
            .count();
        let fixture_rows = rows
            .iter()
            .filter(|row| {
                matches!(
                    row.source_class,
                    ComparableEvidenceSourceClass::FixtureArchitectureTest
                        | ComparableEvidenceSourceClass::SyntheticTest
                )
            })
            .count();
        let row_level_count = rows.iter().filter(|row| row.row_level).count();
        let summary_derived_count = rows.iter().filter(|row| row.summary_derived).count();
        let outcome_reference_count = rows
            .iter()
            .filter(|row| row.outcome_reference_available)
            .count();
        let baseline_reference_count = rows
            .iter()
            .filter(|row| row.baseline_reference_available)
            .count();
        let no_trade_counterfactual_count = rows
            .iter()
            .filter(|row| row.no_trade_counterfactual_available)
            .count();
        let risk_denied_counterfactual_count = rows
            .iter()
            .filter(|row| row.risk_denied_counterfactual_available)
            .count();
        let external_reference_count = rows
            .iter()
            .filter(|row| row.external_reference_available)
            .count();
        let no_lookahead_safe_count = rows.iter().filter(|row| row.no_lookahead_safe).count();
        let storage_bytes = serde_json::to_vec(&rows)
            .map(|bytes| bytes.len())
            .unwrap_or_default();
        Self {
            comparable_id: config.comparable_id.clone(),
            rows,
            complete_rows,
            incomplete_rows,
            official_rows,
            non_crypto_official_rows,
            crypto_only_rows,
            controlled_rows,
            yfinance_rows,
            fixture_rows,
            row_level_count,
            summary_derived_count,
            outcome_reference_count,
            baseline_reference_count,
            no_trade_counterfactual_count,
            risk_denied_counterfactual_count,
            external_reference_count,
            no_lookahead_safe_count,
            storage_bytes,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly])
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("comparable_id={}", self.comparable_id),
            format!("row_count={}", self.rows.len()),
            format!("complete_rows={}", self.complete_rows),
            format!("incomplete_rows={}", self.incomplete_rows),
            format!("official_rows={}", self.official_rows),
            format!("non_crypto_official_rows={}", self.non_crypto_official_rows),
            format!("crypto_only_rows={}", self.crypto_only_rows),
            format!("controlled_rows={}", self.controlled_rows),
            format!("yfinance_rows={}", self.yfinance_rows),
            format!("fixture_rows={}", self.fixture_rows),
            format!("row_level_count={}", self.row_level_count),
            format!("summary_derived_count={}", self.summary_derived_count),
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
            format!("external_reference_count={}", self.external_reference_count),
            format!("no_lookahead_safe_count={}", self.no_lookahead_safe_count),
            format!("storage_bytes={}", self.storage_bytes),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.rows.iter().map(|row| {
            format!(
                "row_id={};symbol={};timestamp_ms={};source_class={:?};row_level={};summary_derived={};complete={};official_readiness_eligible={};diagnostic_only={};candle_coverage_available={};matched_candle_series_id={};candle_match_status={}",
                row.row_id,
                row.symbol,
                row.timestamp_ms,
                row.source_class,
                row.row_level,
                row.summary_derived,
                row.no_lookahead_safe
                    && row.outcome_reference_available
                    && row.baseline_reference_available,
                row.official_readiness_eligible,
                row.diagnostic_only,
                row.candle_coverage_available,
                row.matched_candle_series_id.clone().unwrap_or_default(),
                row.candle_match_status.clone().unwrap_or_default(),
            )
        }));
        lines.join("\n")
    }

    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self)
                .unwrap_or_else(|_| format!("{}:{}", self.comparable_id, self.rows.len())),
        )
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
            output_dir.join("comparable_evidence_bundle.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("comparable_evidence_bundle.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn classify_source_class_for_row(row: &CommitteeScenarioRow) -> ComparableEvidenceSourceClass {
    classify_source_class_from_boundary(classify_row_boundary(row), row)
}

pub fn classify_source_class_from_boundary(
    boundary: OfficialEvidenceBoundary,
    row: &CommitteeScenarioRow,
) -> ComparableEvidenceSourceClass {
    match boundary {
        OfficialEvidenceBoundary::OfficialNonCrypto => {
            ComparableEvidenceSourceClass::OfficialNonCrypto
        }
        OfficialEvidenceBoundary::OfficialCryptoOnly => {
            ComparableEvidenceSourceClass::OfficialCryptoOnly
        }
        OfficialEvidenceBoundary::ResearchOnly => ComparableEvidenceSourceClass::YFinanceResearch,
        OfficialEvidenceBoundary::FixtureOnly => {
            if row.source_kind == CommitteeScenarioSourceKind::SyntheticTest {
                ComparableEvidenceSourceClass::SyntheticTest
            } else {
                ComparableEvidenceSourceClass::FixtureArchitectureTest
            }
        }
        OfficialEvidenceBoundary::Controlled => ComparableEvidenceSourceClass::ControlledDiagnostic,
        OfficialEvidenceBoundary::Unknown => match row.evidence_source_kind {
            EvidenceSourceKind::YFinanceResearch => ComparableEvidenceSourceClass::YFinanceResearch,
            EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::TestFixture => {
                ComparableEvidenceSourceClass::FixtureArchitectureTest
            }
            EvidenceSourceKind::GeneratedSynthetic => ComparableEvidenceSourceClass::SyntheticTest,
            EvidenceSourceKind::OfficialApiCollected if row.market == ProviderMarket::Crypto => {
                ComparableEvidenceSourceClass::OfficialCryptoOnly
            }
            EvidenceSourceKind::OfficialApiCollected => {
                ComparableEvidenceSourceClass::OfficialNonCrypto
            }
            EvidenceSourceKind::RealLocal => ComparableEvidenceSourceClass::ControlledDiagnostic,
            _ => ComparableEvidenceSourceClass::Unknown,
        },
    }
}

pub fn timeframe_for_row(row: &CommitteeScenarioRow) -> String {
    match row.target_horizon {
        PersonaHorizon::Intraday => "intraday",
        PersonaHorizon::Swing => "swing",
        PersonaHorizon::MultiDay => "multiday",
        PersonaHorizon::LongTerm => "longterm",
    }
    .to_string()
}

pub fn horizon_bars_for_row(row: &CommitteeScenarioRow) -> usize {
    match row.target_horizon {
        PersonaHorizon::Intraday => 6,
        PersonaHorizon::Swing => 24,
        PersonaHorizon::MultiDay => 48,
        PersonaHorizon::LongTerm => 96,
    }
}

pub fn infer_market_from_symbol(symbol: &str) -> ProviderMarket {
    let upper = symbol.to_ascii_uppercase();
    if upper.contains("BTC")
        || upper.contains("ETH")
        || upper.contains("USDT")
        || upper.contains("KRW")
    {
        ProviderMarket::Crypto
    } else if upper.chars().all(|ch| ch.is_ascii_digit()) {
        ProviderMarket::KoreanEquity
    } else {
        ProviderMarket::USEquity
    }
}

fn default_output_root() -> String {
    "target/soma_comparable_committee_evidence".to_string()
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
