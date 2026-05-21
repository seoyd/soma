use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SignalQualityInputs {
    pub evaluated_rows: usize,
    pub official_evaluated_rows: usize,
    pub outcome_linked_rows: usize,
    pub baseline_reference_rows: usize,
    pub external_reference_rows: usize,
    pub committee_decision_rows: usize,
    pub baseline_action_rows: usize,
    pub no_trade_rows: usize,
    #[serde(default)]
    pub p_win_calibration: Option<f64>,
    #[serde(default)]
    pub brier_score: Option<f64>,
    #[serde(default)]
    pub ece: Option<f64>,
    #[serde(default)]
    pub expected_edge_avg: Option<f64>,
    #[serde(default)]
    pub realized_edge_proxy: Option<f64>,
    #[serde(default)]
    pub net_return_proxy: Option<f64>,
    #[serde(default)]
    pub research_only: bool,
    #[serde(default)]
    pub fixture_only: bool,
    #[serde(default)]
    pub crypto_only: bool,
    #[serde(default)]
    pub controlled_only: bool,
    #[serde(default)]
    pub require_official_for_usefulness_claim: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalQualityStatus {
    HealthySignalEvidence,
    BaselineOnlyEvidence,
    CommitteeEvidenceAvailable,
    ExternalEvidenceAvailable,
    InsufficientOutcomeLinks,
    PoorCalibration,
    NoBaselineReference,
    NoExternalReference,
    ResearchOnly,
    FixtureOnly,
    CryptoOnly,
    #[default]
    EvidenceInsufficient,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalQualityReport {
    pub evaluated_rows: usize,
    pub official_evaluated_rows: usize,
    pub outcome_linked_rows: usize,
    pub baseline_reference_rows: usize,
    pub external_reference_rows: usize,
    pub committee_decision_rows: usize,
    pub baseline_action_rows: usize,
    pub no_trade_rows: usize,
    #[serde(default)]
    pub p_win_calibration: Option<f64>,
    #[serde(default)]
    pub brier_score: Option<f64>,
    #[serde(default)]
    pub ece: Option<f64>,
    #[serde(default)]
    pub expected_edge_avg: Option<f64>,
    #[serde(default)]
    pub realized_edge_proxy: Option<f64>,
    #[serde(default)]
    pub net_return_proxy: Option<f64>,
    pub signal_quality_status: SignalQualityStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_signal_quality_report(inputs: &SignalQualityInputs) -> SignalQualityReport {
    let poor_calibration = inputs.brier_score.is_some_and(|value| value > 0.25)
        || inputs.ece.is_some_and(|value| value > 0.10);
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    let signal_quality_status = if inputs.research_only {
        warnings.push(
            "yfinance and research-only evidence cannot justify official usefulness".to_string(),
        );
        SignalQualityStatus::ResearchOnly
    } else if inputs.fixture_only {
        warnings.push("fixture evidence remains architecture-test-only".to_string());
        SignalQualityStatus::FixtureOnly
    } else if inputs.crypto_only {
        warnings
            .push("crypto-only evidence cannot satisfy non-crypto official usefulness".to_string());
        SignalQualityStatus::CryptoOnly
    } else if inputs.outcome_linked_rows == 0 {
        blockers.push("outcome links are missing, so usefulness is unproven".to_string());
        SignalQualityStatus::EvidenceInsufficient
    } else if poor_calibration {
        blockers.push("signal calibration remains too weak for a usefulness claim".to_string());
        SignalQualityStatus::PoorCalibration
    } else if inputs.baseline_reference_rows == 0 {
        blockers.push("baseline references are missing".to_string());
        SignalQualityStatus::NoBaselineReference
    } else if inputs.committee_decision_rows == 0 {
        warnings.push("only baseline evidence is available".to_string());
        SignalQualityStatus::BaselineOnlyEvidence
    } else if inputs.external_reference_rows > 0 {
        SignalQualityStatus::ExternalEvidenceAvailable
    } else if inputs.official_evaluated_rows > 0 && inputs.committee_decision_rows > 0 {
        SignalQualityStatus::CommitteeEvidenceAvailable
    } else if inputs.official_evaluated_rows > 0 {
        SignalQualityStatus::HealthySignalEvidence
    } else {
        SignalQualityStatus::EvidenceInsufficient
    };

    if inputs.controlled_only {
        warnings.push(
            "controlled evidence is diagnostic-only even when internally consistent".to_string(),
        );
    }
    if inputs.require_official_for_usefulness_claim && inputs.official_evaluated_rows == 0 {
        blockers.push(
            "non-crypto official evidence is still required for usefulness claims".to_string(),
        );
    }
    if inputs.external_reference_rows == 0 && inputs.committee_decision_rows > 0 {
        warnings.push("external reference path is unavailable".to_string());
    }

    let mut reason_codes = inputs.reason_codes.clone();
    reason_codes.push(ReasonCode::SignalQualityReportBuilt);
    if poor_calibration {
        reason_codes.push(ReasonCode::AiSignalPoorCalibration);
    }
    if inputs.outcome_linked_rows == 0 {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
    }

    SignalQualityReport {
        evaluated_rows: inputs.evaluated_rows,
        official_evaluated_rows: inputs.official_evaluated_rows,
        outcome_linked_rows: inputs.outcome_linked_rows,
        baseline_reference_rows: inputs.baseline_reference_rows,
        external_reference_rows: inputs.external_reference_rows,
        committee_decision_rows: inputs.committee_decision_rows,
        baseline_action_rows: inputs.baseline_action_rows,
        no_trade_rows: inputs.no_trade_rows,
        p_win_calibration: inputs.p_win_calibration,
        brier_score: inputs.brier_score,
        ece: inputs.ece,
        expected_edge_avg: inputs.expected_edge_avg,
        realized_edge_proxy: inputs.realized_edge_proxy,
        net_return_proxy: inputs.net_return_proxy,
        signal_quality_status,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

impl SignalQualityReport {
    pub fn to_text(&self) -> String {
        [
            format!("evaluated_rows={}", self.evaluated_rows),
            format!("official_evaluated_rows={}", self.official_evaluated_rows),
            format!("outcome_linked_rows={}", self.outcome_linked_rows),
            format!("baseline_reference_rows={}", self.baseline_reference_rows),
            format!("external_reference_rows={}", self.external_reference_rows),
            format!("committee_decision_rows={}", self.committee_decision_rows),
            format!("baseline_action_rows={}", self.baseline_action_rows),
            format!("no_trade_rows={}", self.no_trade_rows),
            format!(
                "p_win_calibration={}",
                self.p_win_calibration
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "brier_score={}",
                self.brier_score
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "ece={}",
                self.ece
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "expected_edge_avg={}",
                self.expected_edge_avg
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "realized_edge_proxy={}",
                self.realized_edge_proxy
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "net_return_proxy={}",
                self.net_return_proxy
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!("signal_quality_status={:?}", self.signal_quality_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}
