use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::EvidenceSourceKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeTripleBarrierLabel {
    TakeProfit,
    StopLoss,
    TimeExpired,
    NoTradeCounterfactual,
    RiskDeniedCounterfactual,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeBaselineAction {
    Approve,
    ReduceSize,
    NoTrade,
    Deny,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcomeReference {
    pub outcome_id: String,
    #[serde(default)]
    pub decision_id: Option<String>,
    pub symbol: String,
    pub timestamp_ms: u64,
    pub horizon_bars: usize,
    pub triple_barrier_label: CommitteeTripleBarrierLabel,
    #[serde(default)]
    pub net_return_pct: Option<f64>,
    #[serde(default)]
    pub max_favorable_excursion_pct: Option<f64>,
    #[serde(default)]
    pub max_adverse_excursion_pct: Option<f64>,
    #[serde(default)]
    pub cost_bps: f64,
    #[serde(default)]
    pub slippage_bps: f64,
    pub source_kind: EvidenceSourceKind,
    #[serde(default)]
    pub no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeBaselineReference {
    pub baseline_action: CommitteeBaselineAction,
    #[serde(default)]
    pub baseline_confidence: Option<f64>,
    #[serde(default)]
    pub baseline_expected_edge: Option<f64>,
    #[serde(default)]
    pub baseline_reason_codes: Vec<ReasonCode>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeExternalReference {
    #[serde(default)]
    pub external_action: Option<String>,
    #[serde(default)]
    pub external_p_win: Option<f64>,
    #[serde(default)]
    pub external_confidence: Option<f64>,
    #[serde(default)]
    pub prediction_schema_valid: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeOutcomeReference {
    pub fn cost_adjusted_return_pct(&self) -> Option<f64> {
        self.net_return_pct
            .map(|value| value - (self.cost_bps + self.slippage_bps) / 10_000.0)
    }

    pub fn benchmark_eligible(&self) -> bool {
        self.no_lookahead_safe && self.triple_barrier_label != CommitteeTripleBarrierLabel::Unknown
    }

    pub fn no_trade_counterfactual(&self) -> bool {
        self.triple_barrier_label == CommitteeTripleBarrierLabel::NoTradeCounterfactual
    }

    pub fn risk_denial_counterfactual(&self) -> bool {
        self.triple_barrier_label == CommitteeTripleBarrierLabel::RiskDeniedCounterfactual
    }
}

impl CommitteeBaselineAction {
    pub fn from_summary(summary: &str) -> Self {
        let lowered = summary.trim().to_ascii_lowercase();
        if lowered.contains("reduce") {
            Self::ReduceSize
        } else if lowered.contains("deny") || lowered.contains("reject") {
            Self::Deny
        } else if lowered.contains("no") && lowered.contains("trade") {
            Self::NoTrade
        } else if lowered.contains("approve")
            || lowered.contains("long")
            || lowered.contains("buy")
            || lowered.contains("candidate")
        {
            Self::Approve
        } else {
            Self::Unknown
        }
    }

    pub fn as_summary_str(self) -> &'static str {
        match self {
            Self::Approve => "Approve",
            Self::ReduceSize => "ReduceSize",
            Self::NoTrade => "NoTrade",
            Self::Deny => "Deny",
            Self::Unknown => "Unknown",
        }
    }

    pub fn sizing_multiplier(self) -> f64 {
        match self {
            Self::Approve => 1.0,
            Self::ReduceSize => 0.5,
            Self::NoTrade | Self::Deny | Self::Unknown => 0.0,
        }
    }
}

impl CommitteeExternalReference {
    pub fn action_as_baseline_action(&self) -> CommitteeBaselineAction {
        self.external_action
            .as_deref()
            .map(CommitteeBaselineAction::from_summary)
            .unwrap_or(CommitteeBaselineAction::Unknown)
    }
}

pub fn parse_evidence_source_kind(raw: Option<&str>) -> EvidenceSourceKind {
    match raw.unwrap_or_default() {
        "RealLocal" => EvidenceSourceKind::RealLocal,
        "OfficialApiCollected" => EvidenceSourceKind::OfficialApiCollected,
        "YFinanceResearch" => EvidenceSourceKind::YFinanceResearch,
        "SyntheticFixture" => EvidenceSourceKind::SyntheticFixture,
        "TestFixture" => EvidenceSourceKind::TestFixture,
        "GeneratedSynthetic" => EvidenceSourceKind::GeneratedSynthetic,
        "ExternalPredictionOnly" => EvidenceSourceKind::ExternalPredictionOnly,
        _ => EvidenceSourceKind::Unknown,
    }
}

pub fn parse_triple_barrier_label(raw: Option<&str>) -> CommitteeTripleBarrierLabel {
    match raw.unwrap_or_default() {
        "TakeProfit" => CommitteeTripleBarrierLabel::TakeProfit,
        "StopLoss" => CommitteeTripleBarrierLabel::StopLoss,
        "TimeExpired" => CommitteeTripleBarrierLabel::TimeExpired,
        "NoTradeCounterfactual" => CommitteeTripleBarrierLabel::NoTradeCounterfactual,
        "RiskDeniedCounterfactual" => CommitteeTripleBarrierLabel::RiskDeniedCounterfactual,
        _ => CommitteeTripleBarrierLabel::Unknown,
    }
}
