use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{
    CoreCheckConfig, CoreCheckRunner, MarketSnapshot, ReasonCode, Regime, RiskSnapshot,
    SignalOutput,
};
use crate::data::{EvidenceSourceKind, ProviderMarket};
use crate::feature::FeatureVector;

use super::committee_smoke::CommitteeSmokeTestConfig;
use super::persona_card_lite::PersonaHorizon;
use super::persona_scorer::PersonaScoringInput;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeScenarioSourceKind {
    Fixture,
    EvidenceLaneReport,
    SourceAwareBenchmarkReport,
    YahooResearchEvidenceReport,
    OfficialBenchmarkReport,
    CoreCheckedBenchmarkReport,
    SyntheticTest,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeScenarioLoadConfig {
    pub scenario_id: String,
    pub source_kind: CommitteeScenarioSourceKind,
    #[serde(default)]
    pub input_paths: Vec<String>,
    pub output_root: String,
    #[serde(default = "default_max_scenarios")]
    pub max_scenarios: usize,
    #[serde(default)]
    pub require_core_check: bool,
    #[serde(default = "default_true")]
    pub allow_yfinance_research: bool,
    #[serde(default = "default_true")]
    pub allow_fixture: bool,
    #[serde(default)]
    pub allow_synthetic_test: bool,
    #[serde(default = "default_min_quality")]
    pub min_data_quality: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeScenarioRow {
    pub scenario_row_id: String,
    pub symbol: String,
    pub timestamp_ms: u64,
    pub source_kind: CommitteeScenarioSourceKind,
    pub evidence_source_kind: EvidenceSourceKind,
    pub market: ProviderMarket,
    pub target_horizon: PersonaHorizon,
    #[serde(default)]
    pub feature_vector: Option<FeatureVector>,
    pub regime: Regime,
    pub signal_summary: String,
    pub data_quality_score: f64,
    #[serde(default)]
    pub spread_bps: Option<f64>,
    pub expected_edge_after_cost: f64,
    pub expected_drawdown: f64,
    #[serde(default)]
    pub risk_snapshot_summary: Option<String>,
    pub provenance_summary: String,
    #[serde(default)]
    pub benchmark_status: Option<String>,
    #[serde(default)]
    pub baseline_signal_summary: Option<String>,
    #[serde(default)]
    pub external_prediction_summary: Option<String>,
    #[serde(default)]
    pub no_trade_counterfactual: Option<String>,
    #[serde(default)]
    pub risk_denial_counterfactual: Option<String>,
    #[serde(default)]
    pub outcome_reference: Option<String>,
    #[serde(default)]
    pub materialization_level: CommitteeScenarioMaterializationLevel,
    #[serde(default = "default_materialization_confidence")]
    pub materialization_confidence: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeScenarioMaterializationLevel {
    RowLevel,
    BenchmarkSummary,
    EvidenceSummary,
    Fixture,
    #[default]
    SyntheticSummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeScenarioSet {
    pub scenario_id: String,
    pub rows: Vec<CommitteeScenarioRow>,
    pub source_summary: String,
    pub row_count: usize,
    pub official_row_count: usize,
    pub research_only_row_count: usize,
    pub fixture_row_count: usize,
    pub skipped_row_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeScenarioLoader;

impl Default for CommitteeScenarioLoadConfig {
    fn default() -> Self {
        Self {
            scenario_id: "committee_scenarios".to_string(),
            source_kind: CommitteeScenarioSourceKind::Fixture,
            input_paths: Vec::new(),
            output_root: "target/soma_committee_scenarios".to_string(),
            max_scenarios: default_max_scenarios(),
            require_core_check: false,
            allow_yfinance_research: true,
            allow_fixture: true,
            allow_synthetic_test: false,
            min_data_quality: default_min_quality(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
        }
    }
}

impl CommitteeScenarioLoadConfig {
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

    pub fn validate_local_paths(&self) -> Result<(), String> {
        if self.output_root.contains("://")
            || self.input_paths.iter().any(|path| path.contains("://"))
        {
            return Err("committee scenario loader paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.scenario_id)
    }

    pub fn from_committee_smoke_config(config: &CommitteeSmokeTestConfig) -> Self {
        let mut input_paths = Vec::new();
        if let Some(path) = &config.evidence_plan_path {
            input_paths.push(path.clone());
        }
        if let Some(path) = &config.source_benchmark_report_path {
            input_paths.push(path.clone());
        }
        if let Some(path) = &config.yfinance_report_path {
            input_paths.push(path.clone());
        }
        let source_kind =
            if config.yfinance_report_path.is_some() || config.use_yfinance_research_lane {
                CommitteeScenarioSourceKind::YahooResearchEvidenceReport
            } else if config.source_benchmark_report_path.is_some() {
                CommitteeScenarioSourceKind::SourceAwareBenchmarkReport
            } else if config.evidence_plan_path.is_some() {
                CommitteeScenarioSourceKind::EvidenceLaneReport
            } else if config.use_fixture_data {
                CommitteeScenarioSourceKind::Fixture
            } else {
                CommitteeScenarioSourceKind::Unknown
            };
        Self {
            scenario_id: format!("{}-scenario-set", config.test_id),
            source_kind,
            input_paths,
            output_root: config.output_root.clone(),
            max_scenarios: config.max_decisions,
            require_core_check: config.require_core_check,
            allow_yfinance_research: true,
            allow_fixture: config.use_fixture_data,
            allow_synthetic_test: false,
            min_data_quality: 0.70,
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
        }
    }
}

impl CommitteeScenarioRow {
    pub fn to_scoring_input(&self) -> PersonaScoringInput {
        PersonaScoringInput {
            symbol: self.symbol.clone(),
            timestamp_ms: self.timestamp_ms,
            source_kind: self.evidence_source_kind,
            market: self.market,
            target_horizon: self.target_horizon,
            feature_vector: self.feature_vector.clone(),
            regime: self.regime,
            signal_output: SignalOutput {
                symbol: self.symbol.clone(),
                horizon_bars: match self.target_horizon {
                    PersonaHorizon::Intraday => 6,
                    PersonaHorizon::Swing => 24,
                    PersonaHorizon::MultiDay => 48,
                    PersonaHorizon::LongTerm => 96,
                },
                p_win: 0.60,
                p_stop: 0.28,
                expected_return: self.expected_edge_after_cost,
                expected_drawdown: self.expected_drawdown,
                confidence: (self.data_quality_score * 0.8).clamp(0.0, 1.0),
                no_trade_probability: (1.0 - self.data_quality_score * 0.8).clamp(0.0, 1.0),
                source: self.signal_summary.clone(),
            },
            data_quality_score: self.data_quality_score,
            spread_bps: self.spread_bps,
            expected_edge_after_cost: self.expected_edge_after_cost,
            expected_drawdown: self.expected_drawdown,
            risk_snapshot: Some(self.to_risk_snapshot()),
            reason_codes: self.reason_codes.clone(),
        }
    }

    pub fn to_market_snapshot(&self) -> MarketSnapshot {
        let price = if self.symbol.starts_with("BTC") {
            100_000_000.0
        } else {
            190.0
        };
        let spread_bps = self.spread_bps.unwrap_or(6.0);
        let half_spread = price * spread_bps / 20_000.0;
        MarketSnapshot {
            symbol: self.symbol.clone(),
            timestamp_ms: self.timestamp_ms,
            price,
            bid: price - half_spread,
            ask: price + half_spread,
            spread_bps,
            volume: 10_000.0,
            trade_value: 1_000_000.0,
            volatility: self.expected_drawdown,
            regime: self.regime,
            data_quality_score: self.data_quality_score,
        }
    }

    pub fn to_risk_snapshot(&self) -> RiskSnapshot {
        RiskSnapshot {
            daily_pnl_pct: 0.0,
            consecutive_losses: 0,
            current_positions_count: 0,
            total_exposure_pct: 0.0,
            symbol_exposure_pct: 0.0,
            api_health_score: if self.evidence_source_kind == EvidenceSourceKind::YFinanceResearch {
                0.9
            } else {
                1.0
            },
            data_quality_score: self.data_quality_score,
        }
    }
}

impl CommitteeScenarioSet {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("scenario_id={}", self.scenario_id),
            format!("source_summary={}", self.source_summary),
            format!("row_count={}", self.row_count),
            format!("official_row_count={}", self.official_row_count),
            format!("research_only_row_count={}", self.research_only_row_count),
            format!("fixture_row_count={}", self.fixture_row_count),
            format!("skipped_row_count={}", self.skipped_row_count),
        ];
        for row in &self.rows {
            lines.push(format!(
                "row={};source={:?};evidence={:?};symbol={};quality={:.3};benchmark_status={}",
                row.scenario_row_id,
                row.source_kind,
                row.evidence_source_kind,
                row.symbol,
                row.data_quality_score,
                row.benchmark_status.clone().unwrap_or_default()
            ));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("committee_scenario_set.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_scenario_set.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl CommitteeScenarioLoader {
    pub fn load(
        &self,
        config: &CommitteeScenarioLoadConfig,
    ) -> Result<CommitteeScenarioSet, String> {
        config.validate_local_paths()?;
        if config.require_core_check {
            let _ = CoreCheckRunner::default().run(&CoreCheckConfig::default())?;
        }
        let mut rows = match config.source_kind {
            CommitteeScenarioSourceKind::Fixture => build_fixture_rows(config),
            CommitteeScenarioSourceKind::EvidenceLaneReport => load_evidence_lane_rows(config)?,
            CommitteeScenarioSourceKind::SourceAwareBenchmarkReport => {
                load_source_benchmark_rows(config)?
            }
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport => load_yfinance_rows(config)?,
            CommitteeScenarioSourceKind::OfficialBenchmarkReport
            | CommitteeScenarioSourceKind::CoreCheckedBenchmarkReport => {
                load_official_benchmark_rows(config)?
            }
            CommitteeScenarioSourceKind::SyntheticTest => build_synthetic_rows(config),
            CommitteeScenarioSourceKind::Unknown => Vec::new(),
        };

        let mut skipped_row_count = 0usize;
        rows.retain(|row| {
            let allowed = match row.source_kind {
                CommitteeScenarioSourceKind::Fixture => config.allow_fixture,
                CommitteeScenarioSourceKind::SyntheticTest => config.allow_synthetic_test,
                _ => true,
            } && (config.allow_yfinance_research
                || row.evidence_source_kind != EvidenceSourceKind::YFinanceResearch)
                && row.data_quality_score >= config.min_data_quality;
            if !allowed {
                skipped_row_count += 1;
            }
            allowed
        });

        let mut reason_codes = vec![ReasonCode::CommitteeScenarioLoaderBuilt];
        rows.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
        if rows.len() > config.max_scenarios {
            rows.truncate(config.max_scenarios);
            reason_codes.push(ReasonCode::CommitteeScenarioRowsTruncated);
        }

        let mut sources = BTreeSet::new();
        let mut official_row_count = 0usize;
        let mut research_only_row_count = 0usize;
        let mut fixture_row_count = 0usize;
        for row in &rows {
            sources.insert(format!("{:?}", row.source_kind));
            if row.evidence_source_kind.readiness_eligible() {
                official_row_count += 1;
            }
            if row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch {
                research_only_row_count += 1;
            }
            if matches!(
                row.source_kind,
                CommitteeScenarioSourceKind::Fixture | CommitteeScenarioSourceKind::SyntheticTest
            ) {
                fixture_row_count += 1;
            }
        }

        Ok(CommitteeScenarioSet {
            scenario_id: config.scenario_id.clone(),
            row_count: rows.len(),
            source_summary: sources.into_iter().collect::<Vec<_>>().join("|"),
            official_row_count,
            research_only_row_count,
            fixture_row_count,
            skipped_row_count,
            rows,
            reason_codes: reason_codes
                .into_iter()
                .chain(config.reason_codes.iter().cloned())
                .collect(),
        })
    }
}

fn build_fixture_rows(config: &CommitteeScenarioLoadConfig) -> Vec<CommitteeScenarioRow> {
    vec![
        scenario_row(
            config,
            0,
            CommitteeScenarioSourceKind::Fixture,
            EvidenceSourceKind::TestFixture,
            "BTC-KRW",
            ProviderMarket::Crypto,
            PersonaHorizon::Swing,
            Regime::TrendUp,
            0.90,
            Some(8.0),
            0.012,
            0.020,
            "fixture-summary".to_string(),
            Some("fixture".to_string()),
            vec![ReasonCode::SummaryDerived],
        ),
        scenario_row(
            config,
            1,
            CommitteeScenarioSourceKind::Fixture,
            EvidenceSourceKind::TestFixture,
            "BTC-KRW",
            ProviderMarket::Crypto,
            PersonaHorizon::Swing,
            Regime::Range,
            0.82,
            Some(12.0),
            0.001,
            0.030,
            "fixture-summary".to_string(),
            Some("fixture".to_string()),
            vec![ReasonCode::SummaryDerived],
        ),
    ]
}

fn build_synthetic_rows(config: &CommitteeScenarioLoadConfig) -> Vec<CommitteeScenarioRow> {
    vec![scenario_row(
        config,
        0,
        CommitteeScenarioSourceKind::SyntheticTest,
        EvidenceSourceKind::SyntheticFixture,
        "BTC-KRW",
        ProviderMarket::Crypto,
        PersonaHorizon::Swing,
        Regime::TrendUp,
        0.80,
        Some(10.0),
        0.005,
        0.025,
        "synthetic-summary".to_string(),
        Some("synthetic".to_string()),
        vec![ReasonCode::SummaryDerived],
    )]
}

fn load_evidence_lane_rows(
    config: &CommitteeScenarioLoadConfig,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let mut rows = Vec::new();
    for path in &config.input_paths {
        let value = load_json(path)?;
        for (index, lane) in value
            .get("runnable_lanes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let lane_kind = lane
                .get("lane_kind")
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            let (symbol, market, evidence_source_kind, horizon) = match lane_kind {
                "CryptoIntradayEvidence" | "CryptoEodEvidence" => (
                    "BTC-KRW".to_string(),
                    ProviderMarket::Crypto,
                    EvidenceSourceKind::OfficialApiCollected,
                    PersonaHorizon::Swing,
                ),
                "YFinanceResearchFallback" => (
                    "AAPL".to_string(),
                    ProviderMarket::USEquity,
                    EvidenceSourceKind::YFinanceResearch,
                    PersonaHorizon::Swing,
                ),
                _ => (
                    "AAPL".to_string(),
                    ProviderMarket::USEquity,
                    EvidenceSourceKind::OfficialApiCollected,
                    PersonaHorizon::Swing,
                ),
            };
            rows.push(scenario_row(
                config,
                index,
                CommitteeScenarioSourceKind::EvidenceLaneReport,
                evidence_source_kind,
                &symbol,
                market,
                horizon,
                Regime::TrendUp,
                0.88,
                Some(6.0),
                if evidence_source_kind == EvidenceSourceKind::YFinanceResearch {
                    0.004
                } else {
                    0.008
                },
                0.020,
                format!("evidence-lane:{lane_kind}"),
                lane.get("lane_status")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                vec![ReasonCode::SummaryDerived],
            ));
        }
    }
    Ok(rows)
}

fn load_source_benchmark_rows(
    config: &CommitteeScenarioLoadConfig,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    if config.input_paths.is_empty() {
        return Ok(vec![scenario_row(
            config,
            0,
            CommitteeScenarioSourceKind::SourceAwareBenchmarkReport,
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            ProviderMarket::USEquity,
            PersonaHorizon::Swing,
            Regime::TrendUp,
            0.90,
            Some(5.0),
            0.008,
            0.020,
            "source-benchmark-official".to_string(),
            Some("summary-derived".to_string()),
            vec![ReasonCode::SummaryDerived],
        )]);
    }
    let mut rows = Vec::new();
    for path in &config.input_paths {
        let value = load_json(path)?;
        let official_ready_count = value
            .get("dataset_inventory")
            .and_then(|inventory| inventory.get("official_ready_count"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let yfinance_ready_count = value
            .get("dataset_inventory")
            .and_then(|inventory| inventory.get("yfinance_benchmark_eligible_count"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if official_ready_count > 0 {
            rows.push(scenario_row(
                config,
                rows.len(),
                CommitteeScenarioSourceKind::SourceAwareBenchmarkReport,
                EvidenceSourceKind::OfficialApiCollected,
                "AAPL",
                ProviderMarket::USEquity,
                PersonaHorizon::Swing,
                Regime::TrendUp,
                0.90,
                Some(5.0),
                0.008,
                0.020,
                "source-benchmark-official".to_string(),
                Some("official-ready".to_string()),
                vec![ReasonCode::SummaryDerived],
            ));
        }
        if yfinance_ready_count > 0 {
            rows.push(scenario_row(
                config,
                rows.len(),
                CommitteeScenarioSourceKind::SourceAwareBenchmarkReport,
                EvidenceSourceKind::YFinanceResearch,
                "AAPL",
                ProviderMarket::USEquity,
                PersonaHorizon::Swing,
                Regime::Range,
                0.86,
                Some(6.0),
                0.004,
                0.020,
                "source-benchmark-yfinance".to_string(),
                Some("research-only".to_string()),
                vec![ReasonCode::SummaryDerived],
            ));
        }
    }
    Ok(rows)
}

fn load_yfinance_rows(
    config: &CommitteeScenarioLoadConfig,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    if config.input_paths.is_empty() {
        return Ok(vec![scenario_row(
            config,
            0,
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
            EvidenceSourceKind::YFinanceResearch,
            "AAPL",
            ProviderMarket::USEquity,
            PersonaHorizon::Swing,
            Regime::Range,
            0.86,
            Some(6.0),
            0.004,
            0.020,
            "yfinance-summary".to_string(),
            Some("research-only".to_string()),
            vec![ReasonCode::SummaryDerived, ReasonCode::YFinanceResearchOnly],
        )]);
    }
    let mut rows = Vec::new();
    for path in &config.input_paths {
        let value = load_json(path)?;
        let symbols = value
            .get("yfinance_symbols")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for symbol in symbols.iter().filter_map(Value::as_str) {
            rows.push(scenario_row(
                config,
                rows.len(),
                CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
                EvidenceSourceKind::YFinanceResearch,
                symbol,
                ProviderMarket::USEquity,
                PersonaHorizon::Swing,
                Regime::Range,
                0.86,
                Some(6.0),
                0.004,
                0.020,
                "yfinance-summary".to_string(),
                Some("research-only".to_string()),
                vec![ReasonCode::SummaryDerived, ReasonCode::YFinanceResearchOnly],
            ));
        }
        if rows.is_empty() {
            rows.push(scenario_row(
                config,
                0,
                CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
                EvidenceSourceKind::YFinanceResearch,
                "AAPL",
                ProviderMarket::USEquity,
                PersonaHorizon::Swing,
                Regime::Range,
                0.86,
                Some(6.0),
                0.004,
                0.020,
                "yfinance-summary".to_string(),
                Some("research-only".to_string()),
                vec![ReasonCode::SummaryDerived, ReasonCode::YFinanceResearchOnly],
            ));
        }
    }
    Ok(rows)
}

fn load_official_benchmark_rows(
    config: &CommitteeScenarioLoadConfig,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    if config.input_paths.is_empty() {
        return Ok(vec![scenario_row(
            config,
            0,
            config.source_kind,
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            ProviderMarket::USEquity,
            PersonaHorizon::Swing,
            Regime::TrendUp,
            0.91,
            Some(5.0),
            0.009,
            0.020,
            "official-benchmark-summary".to_string(),
            Some("summary-derived".to_string()),
            vec![ReasonCode::SummaryDerived],
        )]);
    }
    let mut rows = Vec::new();
    for path in &config.input_paths {
        let value = load_json(path)?;
        let status = value
            .get("final_status")
            .and_then(Value::as_str)
            .unwrap_or("summary-derived");
        rows.push(scenario_row(
            config,
            rows.len(),
            config.source_kind,
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            ProviderMarket::USEquity,
            PersonaHorizon::Swing,
            Regime::TrendUp,
            0.91,
            Some(5.0),
            0.009,
            0.020,
            "official-benchmark-summary".to_string(),
            Some(status.to_string()),
            vec![ReasonCode::SummaryDerived],
        ));
    }
    Ok(rows)
}

fn load_json(path: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn scenario_row(
    config: &CommitteeScenarioLoadConfig,
    index: usize,
    source_kind: CommitteeScenarioSourceKind,
    evidence_source_kind: EvidenceSourceKind,
    symbol: &str,
    market: ProviderMarket,
    target_horizon: PersonaHorizon,
    regime: Regime,
    data_quality_score: f64,
    spread_bps: Option<f64>,
    expected_edge_after_cost: f64,
    expected_drawdown: f64,
    signal_summary: String,
    benchmark_status: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> CommitteeScenarioRow {
    CommitteeScenarioRow {
        scenario_row_id: format!("{}-row-{index:03}", config.scenario_id),
        symbol: symbol.to_string(),
        timestamp_ms: 1_700_000_000_000 + index as u64,
        source_kind,
        evidence_source_kind,
        market,
        target_horizon,
        feature_vector: None,
        regime,
        signal_summary,
        data_quality_score,
        spread_bps,
        expected_edge_after_cost,
        expected_drawdown,
        risk_snapshot_summary: Some(format!(
            "edge={expected_edge_after_cost:.4};drawdown={expected_drawdown:.4}"
        )),
        provenance_summary: provenance_summary(source_kind, evidence_source_kind),
        benchmark_status,
        baseline_signal_summary: Some("NoTradeBaseline".to_string()),
        external_prediction_summary: None,
        no_trade_counterfactual: Some("always-no-trade".to_string()),
        risk_denial_counterfactual: Some("risk-denied-counterfactual".to_string()),
        outcome_reference: if matches!(
            source_kind,
            CommitteeScenarioSourceKind::Fixture | CommitteeScenarioSourceKind::SyntheticTest
        ) {
            Some("bounded-fixture-outcome".to_string())
        } else {
            None
        },
        materialization_level: match source_kind {
            CommitteeScenarioSourceKind::Fixture => CommitteeScenarioMaterializationLevel::Fixture,
            CommitteeScenarioSourceKind::SyntheticTest => {
                CommitteeScenarioMaterializationLevel::SyntheticSummary
            }
            CommitteeScenarioSourceKind::OfficialBenchmarkReport
            | CommitteeScenarioSourceKind::CoreCheckedBenchmarkReport => {
                CommitteeScenarioMaterializationLevel::BenchmarkSummary
            }
            CommitteeScenarioSourceKind::SourceAwareBenchmarkReport
            | CommitteeScenarioSourceKind::EvidenceLaneReport
            | CommitteeScenarioSourceKind::YahooResearchEvidenceReport
            | CommitteeScenarioSourceKind::Unknown => {
                CommitteeScenarioMaterializationLevel::EvidenceSummary
            }
        },
        materialization_confidence: match source_kind {
            CommitteeScenarioSourceKind::Fixture => 0.95,
            CommitteeScenarioSourceKind::SyntheticTest => 0.60,
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport => 0.55,
            CommitteeScenarioSourceKind::OfficialBenchmarkReport
            | CommitteeScenarioSourceKind::CoreCheckedBenchmarkReport => 0.72,
            CommitteeScenarioSourceKind::SourceAwareBenchmarkReport => 0.70,
            CommitteeScenarioSourceKind::EvidenceLaneReport => 0.68,
            CommitteeScenarioSourceKind::Unknown => 0.50,
        },
        reason_codes,
    }
}

fn provenance_summary(
    source_kind: CommitteeScenarioSourceKind,
    evidence_source_kind: EvidenceSourceKind,
) -> String {
    match (source_kind, evidence_source_kind) {
        (CommitteeScenarioSourceKind::Fixture, _) => "fixture-only-summary".to_string(),
        (CommitteeScenarioSourceKind::SyntheticTest, _) => "synthetic-test-summary".to_string(),
        (_, EvidenceSourceKind::YFinanceResearch) => "yfinance-research-summary".to_string(),
        _ => "official-summary-derived".to_string(),
    }
}

fn default_true() -> bool {
    true
}

fn default_max_scenarios() -> usize {
    50
}

fn default_min_quality() -> f64 {
    0.70
}

fn default_materialization_confidence() -> f64 {
    0.50
}
