use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{
    CoreCheckConfig, CoreCheckRunner, MarketSnapshot, ReasonCode, Regime, RiskSnapshot,
    SignalOutput,
};
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::chair_v0::ChairV0;
use super::committee_decision::{
    ChairCommitteeConfig, CommitteeDebateReport, CommitteeDecision, CommitteeInput,
};
use super::committee_evaluation::{
    CommitteeEvaluationScaffold, build_committee_evaluation_scaffold,
};
use super::committee_risk_bridge::{CommitteeFinalAction, CommitteeOutcome, CommitteeRiskBridge};
use super::persona_card_lite::{PersonaCardLite, PersonaHorizon, active_persona_cards_lite};
use super::persona_scorer::PersonaScoringInput;
use super::trinity_personas::active_trinity_scorers;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeSmokeTestConfig {
    pub test_id: String,
    #[serde(default)]
    pub evidence_plan_path: Option<String>,
    #[serde(default)]
    pub source_benchmark_report_path: Option<String>,
    #[serde(default)]
    pub yfinance_report_path: Option<String>,
    #[serde(default = "default_true")]
    pub use_fixture_data: bool,
    #[serde(default)]
    pub use_upbit_crypto_lane: bool,
    #[serde(default)]
    pub use_yfinance_research_lane: bool,
    pub output_root: String,
    #[serde(default = "default_max_decisions")]
    pub max_decisions: usize,
    #[serde(default = "default_true")]
    pub require_core_check: bool,
    #[serde(default)]
    pub force_core_block: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeSmokeFinalStatus {
    CommitteeSmokePassed,
    CommitteeAllNoTrade,
    CommitteeRiskBlocked,
    CommitteeCoreBlocked,
    CommitteeDataInsufficient,
    CommitteeResearchOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeSmokeRecommendation {
    KeepTrinity,
    ImproveChairFirst,
    ImprovePersonaScoringFirst,
    ImproveRiskGovernorFirst,
    ExpandToSixDesignReview,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeSmokeTestReport {
    pub test_id: String,
    pub source_summary: String,
    pub active_personas: Vec<PersonaCardLite>,
    pub decisions: Vec<CommitteeOutcome>,
    pub chair_summary: Vec<String>,
    pub risk_summary: Vec<String>,
    pub debate_summary: CommitteeDebateReport,
    pub evaluation_scaffold: CommitteeEvaluationScaffold,
    pub final_status: CommitteeSmokeFinalStatus,
    pub final_recommendation: CommitteeSmokeRecommendation,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeSmokeTestRunner;

impl Default for CommitteeSmokeTestConfig {
    fn default() -> Self {
        Self {
            test_id: "committee_smoke".to_string(),
            evidence_plan_path: None,
            source_benchmark_report_path: None,
            yfinance_report_path: None,
            use_fixture_data: true,
            use_upbit_crypto_lane: true,
            use_yfinance_research_lane: false,
            output_root: "target/soma_committee_smoke".to_string(),
            max_decisions: default_max_decisions(),
            require_core_check: true,
            force_core_block: false,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
        }
    }
}

impl CommitteeSmokeTestConfig {
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
        for value in [
            Some(self.output_root.as_str()),
            self.evidence_plan_path.as_deref(),
            self.source_benchmark_report_path.as_deref(),
            self.yfinance_report_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if value.contains("://") {
                return Err("committee-smoke config path must be local".to_string());
            }
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.test_id)
    }
}

impl CommitteeSmokeTestRunner {
    pub fn run(
        &self,
        config: &CommitteeSmokeTestConfig,
    ) -> Result<CommitteeSmokeTestReport, String> {
        config.validate_local_paths()?;
        if config.force_core_block {
            return Ok(CommitteeSmokeTestReport {
                test_id: config.test_id.clone(),
                source_summary: "core-check blocked".to_string(),
                active_personas: active_persona_cards_lite(),
                decisions: vec![],
                chair_summary: vec![],
                risk_summary: vec![],
                debate_summary: empty_debate(),
                evaluation_scaffold: build_committee_evaluation_scaffold(&[]),
                final_status: CommitteeSmokeFinalStatus::CommitteeCoreBlocked,
                final_recommendation: CommitteeSmokeRecommendation::ImproveRiskGovernorFirst,
                warnings: vec!["core-check blocked committee smoke".to_string()],
                reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            });
        }
        if config.require_core_check {
            let _ = CoreCheckRunner::default().run(&CoreCheckConfig::default())?;
        }

        let active_personas = active_persona_cards_lite();
        let mut scenarios = build_scenarios(config);
        scenarios.extend(load_configured_scenarios(config)?);
        dedupe_scenarios(&mut scenarios);
        if scenarios.is_empty() {
            return Ok(CommitteeSmokeTestReport {
                test_id: config.test_id.clone(),
                source_summary: "no scenarios".to_string(),
                active_personas,
                decisions: vec![],
                chair_summary: vec![],
                risk_summary: vec![],
                debate_summary: empty_debate(),
                evaluation_scaffold: build_committee_evaluation_scaffold(&[]),
                final_status: CommitteeSmokeFinalStatus::CommitteeDataInsufficient,
                final_recommendation: CommitteeSmokeRecommendation::NeedMoreEvidence,
                warnings: vec!["no fixture or source scenario selected".to_string()],
                reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            });
        }

        let scorers = active_trinity_scorers();
        let chair = ChairV0 {
            config: ChairCommitteeConfig::default(),
        };
        let bridge = CommitteeRiskBridge::default();

        let mut decisions = Vec::new();
        for (market, risk, scoring_input) in scenarios.into_iter().take(config.max_decisions) {
            let persona_votes = scorers
                .iter()
                .map(|scorer| scorer.score(&scoring_input))
                .collect::<Vec<_>>();
            let committee_input = CommitteeInput {
                target_horizon: scoring_input.target_horizon,
                source_kind: scoring_input.source_kind,
                regime: scoring_input.regime,
                scoring_input: scoring_input.clone(),
                persona_votes,
                reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            };
            let record = chair.evaluate(&committee_input);
            decisions.push(bridge.evaluate(&market, &risk, &committee_input.scoring_input, record));
        }
        let debate_summary = build_debate_summary(&decisions);
        let evaluation_scaffold = build_committee_evaluation_scaffold(&decisions);
        let final_status = classify_final_status(&decisions);
        let final_recommendation = classify_recommendation(final_status, &debate_summary);
        let chair_summary = vec![
            format!("decision_count={}", decisions.len()),
            format!(
                "approve_like_count={}",
                decisions
                    .iter()
                    .filter(|outcome| {
                        matches!(
                            outcome.committee_record.final_decision,
                            CommitteeDecision::ApproveCandidate
                                | CommitteeDecision::ReduceSizeCandidate
                        )
                    })
                    .count()
            ),
        ];
        let risk_summary = vec![format!(
            "risk_denied_count={}",
            decisions
                .iter()
                .filter(|outcome| matches!(outcome.final_action, CommitteeFinalAction::FinalDenied))
                .count()
        )];
        let mut warnings = debate_summary.warnings.clone();
        if config.use_yfinance_research_lane {
            warnings.push("yfinance remains research-only".to_string());
        }
        if config.use_upbit_crypto_lane {
            warnings.push("Upbit committee smoke remains crypto-only".to_string());
        }
        warnings.sort();
        warnings.dedup();
        Ok(CommitteeSmokeTestReport {
            test_id: config.test_id.clone(),
            source_summary: build_source_summary(config),
            active_personas,
            decisions,
            chair_summary,
            risk_summary,
            debate_summary,
            evaluation_scaffold,
            final_status,
            final_recommendation,
            warnings,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
        })
    }
}

impl CommitteeSmokeTestReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("test_id={}", self.test_id),
            format!("source_summary={}", self.source_summary),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("warnings={}", self.warnings.join("|")),
        ];
        for persona in &self.active_personas {
            lines.push(format!("active_persona={}", persona.persona_id));
        }
        for outcome in &self.decisions {
            lines.push(format!(
                "decision={};final_decision={:?};final_action={:?}",
                outcome.committee_record.decision_id,
                outcome.committee_record.final_decision,
                outcome.final_action
            ));
        }
        lines.push(self.debate_summary.to_text());
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("committee_smoke_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_smoke_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn build_source_summary(config: &CommitteeSmokeTestConfig) -> String {
    let mut parts = Vec::new();
    if config.use_fixture_data {
        parts.push("fixture".to_string());
    }
    if config.use_upbit_crypto_lane {
        parts.push("upbit-crypto-only".to_string());
    }
    if config.use_yfinance_research_lane {
        parts.push("yfinance-research-only".to_string());
    }
    parts.join("|")
}

fn build_scenarios(
    config: &CommitteeSmokeTestConfig,
) -> Vec<(MarketSnapshot, RiskSnapshot, PersonaScoringInput)> {
    let mut scenarios = Vec::new();
    if config.use_fixture_data || config.use_upbit_crypto_lane {
        scenarios.push(sample_scenario(
            "BTC-KRW",
            EvidenceSourceKind::OfficialApiCollected,
            ProviderMarket::Crypto,
            Regime::TrendUp,
            0.012,
            0.02,
            0.74,
            8.0,
            0.92,
            PersonaHorizon::Swing,
        ));
    }
    if config.use_fixture_data {
        scenarios.push(sample_scenario(
            "BTC-KRW",
            EvidenceSourceKind::OfficialApiCollected,
            ProviderMarket::Crypto,
            Regime::Range,
            0.001,
            0.03,
            0.48,
            12.0,
            0.82,
            PersonaHorizon::Swing,
        ));
    }
    if config.use_yfinance_research_lane {
        scenarios.push(sample_scenario(
            "AAPL",
            EvidenceSourceKind::YFinanceResearch,
            ProviderMarket::USEquity,
            Regime::Range,
            0.004,
            0.02,
            0.60,
            6.0,
            0.88,
            PersonaHorizon::Swing,
        ));
    }
    scenarios
}

fn load_configured_scenarios(
    config: &CommitteeSmokeTestConfig,
) -> Result<Vec<(MarketSnapshot, RiskSnapshot, PersonaScoringInput)>, String> {
    let mut scenarios = Vec::new();
    if let Some(path) = &config.evidence_plan_path {
        let value = load_json(path)?;
        for lane_kind in value
            .get("runnable_lanes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|lane| lane.get("lane_kind").and_then(Value::as_str))
        {
            match lane_kind {
                "CryptoIntradayEvidence" | "CryptoEodEvidence" => scenarios.push(sample_scenario(
                    "BTC-KRW",
                    EvidenceSourceKind::OfficialApiCollected,
                    ProviderMarket::Crypto,
                    Regime::TrendUp,
                    0.012,
                    0.02,
                    0.74,
                    8.0,
                    0.92,
                    PersonaHorizon::Swing,
                )),
                "YFinanceResearchFallback" => scenarios.push(sample_scenario(
                    "AAPL",
                    EvidenceSourceKind::YFinanceResearch,
                    ProviderMarket::USEquity,
                    Regime::Range,
                    0.004,
                    0.02,
                    0.60,
                    6.0,
                    0.88,
                    PersonaHorizon::Swing,
                )),
                "USEquityEodEvidence"
                | "USEquityRealtimeResearch"
                | "USEquityFullMarketRealtimeResearch"
                | "KoreanEquityEodEvidence"
                | "KoreanEquityIntradayResearch" => scenarios.push(sample_scenario(
                    "AAPL",
                    EvidenceSourceKind::OfficialApiCollected,
                    ProviderMarket::USEquity,
                    Regime::TrendUp,
                    0.008,
                    0.02,
                    0.67,
                    5.0,
                    0.90,
                    PersonaHorizon::Swing,
                )),
                _ => {}
            }
        }
    }
    if let Some(path) = &config.source_benchmark_report_path {
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
            scenarios.push(sample_scenario(
                "AAPL",
                EvidenceSourceKind::OfficialApiCollected,
                ProviderMarket::USEquity,
                Regime::TrendUp,
                0.008,
                0.02,
                0.67,
                5.0,
                0.90,
                PersonaHorizon::Swing,
            ));
        }
        if yfinance_ready_count > 0 {
            scenarios.push(sample_scenario(
                "AAPL",
                EvidenceSourceKind::YFinanceResearch,
                ProviderMarket::USEquity,
                Regime::Range,
                0.004,
                0.02,
                0.60,
                6.0,
                0.88,
                PersonaHorizon::Swing,
            ));
        }
    }
    if let Some(path) = &config.yfinance_report_path {
        let value = load_json(path)?;
        let symbol = value
            .get("yfinance_symbols")
            .and_then(Value::as_array)
            .and_then(|symbols| symbols.first())
            .and_then(Value::as_str)
            .unwrap_or("AAPL");
        scenarios.push(sample_scenario(
            symbol,
            EvidenceSourceKind::YFinanceResearch,
            ProviderMarket::USEquity,
            Regime::Range,
            0.004,
            0.02,
            0.60,
            6.0,
            0.88,
            PersonaHorizon::Swing,
        ));
    }
    Ok(scenarios)
}

fn load_json(path: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn dedupe_scenarios(scenarios: &mut Vec<(MarketSnapshot, RiskSnapshot, PersonaScoringInput)>) {
    scenarios.sort_by(|left, right| {
        let lhs = (
            &left.2.symbol,
            left.2.source_kind as u8,
            left.2.market as u8,
        );
        let rhs = (
            &right.2.symbol,
            right.2.source_kind as u8,
            right.2.market as u8,
        );
        lhs.cmp(&rhs)
    });
    scenarios.dedup_by(|left, right| {
        left.2.symbol == right.2.symbol
            && left.2.source_kind == right.2.source_kind
            && left.2.market == right.2.market
    });
}

fn sample_scenario(
    symbol: &str,
    source_kind: EvidenceSourceKind,
    market: ProviderMarket,
    regime: Regime,
    expected_edge_after_cost: f64,
    expected_drawdown: f64,
    confidence: f64,
    spread_bps: f64,
    data_quality_score: f64,
    target_horizon: PersonaHorizon,
) -> (MarketSnapshot, RiskSnapshot, PersonaScoringInput) {
    let market_snapshot = MarketSnapshot {
        symbol: symbol.to_string(),
        timestamp_ms: 1_700_000_000_000,
        price: if symbol == "BTC-KRW" {
            100_000_000.0
        } else {
            190.0
        },
        bid: if symbol == "BTC-KRW" {
            99_950_000.0
        } else {
            189.9
        },
        ask: if symbol == "BTC-KRW" {
            100_050_000.0
        } else {
            190.1
        },
        spread_bps,
        volume: 10_000.0,
        trade_value: 1_000_000.0,
        volatility: expected_drawdown,
        regime,
        data_quality_score,
    };
    let risk_snapshot = RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score,
    };
    let scoring_input = PersonaScoringInput {
        symbol: symbol.to_string(),
        timestamp_ms: market_snapshot.timestamp_ms,
        source_kind,
        market,
        target_horizon,
        feature_vector: None,
        regime,
        signal_output: SignalOutput {
            symbol: symbol.to_string(),
            horizon_bars: if matches!(target_horizon, PersonaHorizon::Intraday) {
                6
            } else {
                24
            },
            p_win: 0.62,
            p_stop: 0.28,
            expected_return: expected_edge_after_cost,
            expected_drawdown,
            confidence,
            no_trade_probability: 1.0 - confidence,
            source: format!("{source_kind:?}"),
        },
        data_quality_score,
        spread_bps: Some(spread_bps),
        expected_edge_after_cost,
        expected_drawdown,
        risk_snapshot: Some(risk_snapshot.clone()),
        reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
    };
    (market_snapshot, risk_snapshot, scoring_input)
}

fn build_debate_summary(outcomes: &[CommitteeOutcome]) -> CommitteeDebateReport {
    let mut selected_speaker_counts = BTreeMap::new();
    let mut disagreement = 0.0;
    for outcome in outcomes {
        for speaker in &outcome.committee_record.selected_speakers {
            *selected_speaker_counts.entry(speaker.clone()).or_insert(0) += 1;
        }
        disagreement += outcome.committee_record.disagreement_score;
    }
    let average_disagreement = disagreement / outcomes.len().max(1) as f64;
    let groupthink_warning = outcomes
        .iter()
        .all(|outcome| outcome.committee_record.groupthink_risk >= 0.65);
    let mut warnings = Vec::new();
    if groupthink_warning {
        warnings.push("GroupthinkWarning".to_string());
        warnings.push("NeedMoreDiversePersonas".to_string());
    }
    CommitteeDebateReport {
        selected_speaker_counts,
        average_disagreement,
        groupthink_warning,
        warnings,
        reason_codes: vec![ReasonCode::CommitteeDebateReportBuilt],
    }
}

fn classify_final_status(outcomes: &[CommitteeOutcome]) -> CommitteeSmokeFinalStatus {
    if outcomes.is_empty() {
        return CommitteeSmokeFinalStatus::CommitteeDataInsufficient;
    }
    if outcomes
        .iter()
        .all(|outcome| outcome.committee_record.source_kind == EvidenceSourceKind::YFinanceResearch)
    {
        return CommitteeSmokeFinalStatus::CommitteeResearchOnly;
    }
    if outcomes
        .iter()
        .all(|outcome| matches!(outcome.final_action, CommitteeFinalAction::FinalDenied))
    {
        return CommitteeSmokeFinalStatus::CommitteeRiskBlocked;
    }
    if outcomes.iter().all(|outcome| {
        matches!(
            outcome.final_action,
            CommitteeFinalAction::FinalNoTrade | CommitteeFinalAction::FinalDenied
        )
    }) {
        CommitteeSmokeFinalStatus::CommitteeAllNoTrade
    } else {
        CommitteeSmokeFinalStatus::CommitteeSmokePassed
    }
}

fn classify_recommendation(
    final_status: CommitteeSmokeFinalStatus,
    debate: &CommitteeDebateReport,
) -> CommitteeSmokeRecommendation {
    match final_status {
        CommitteeSmokeFinalStatus::CommitteeSmokePassed if debate.groupthink_warning => {
            CommitteeSmokeRecommendation::ImproveChairFirst
        }
        CommitteeSmokeFinalStatus::CommitteeSmokePassed => {
            CommitteeSmokeRecommendation::KeepTrinity
        }
        CommitteeSmokeFinalStatus::CommitteeAllNoTrade => {
            CommitteeSmokeRecommendation::ImprovePersonaScoringFirst
        }
        CommitteeSmokeFinalStatus::CommitteeRiskBlocked
        | CommitteeSmokeFinalStatus::CommitteeCoreBlocked => {
            CommitteeSmokeRecommendation::ImproveRiskGovernorFirst
        }
        CommitteeSmokeFinalStatus::CommitteeResearchOnly
        | CommitteeSmokeFinalStatus::CommitteeDataInsufficient => {
            CommitteeSmokeRecommendation::NeedMoreEvidence
        }
    }
}

fn empty_debate() -> CommitteeDebateReport {
    CommitteeDebateReport {
        selected_speaker_counts: BTreeMap::new(),
        average_disagreement: 0.0,
        groupthink_warning: false,
        warnings: vec![],
        reason_codes: vec![ReasonCode::CommitteeDebateReportBuilt],
    }
}

fn default_true() -> bool {
    true
}

fn default_max_decisions() -> usize {
    12
}
