use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoreBottleneckKind {
    MissingOfficialAuth,
    MissingOfficialData,
    MissingOfficialCandles,
    MissingOutcomeLinks,
    MissingBaselineReferences,
    MissingNoTradeCounterfactuals,
    MissingRiskDeniedCounterfactuals,
    PoorCalibration,
    RiskOverBlocking,
    RiskUnderBlocking,
    ChairNeedsTuning,
    PersonaScoringWeak,
    SignalModelWeak,
    ScenarioMaterializationWeak,
    StorageBudgetExceeded,
    LatencyBudgetExceeded,
    EvidenceTooWeak,
    #[default]
    NoBottleneckDetected,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoreBottleneckRecommendation {
    OfficialProviderAuthFirst,
    MoreOfficialEvidence,
    ImproveCandleCoverageFirst,
    ImproveOutcomeLinkingFirst,
    ImproveBaselineReferenceDepth,
    ImproveCounterfactualDepthFirst,
    ImproveRiskGovernorFirst,
    ImproveChairFirst,
    ImprovePersonaScoringFirst,
    ImproveSignalModelFirst,
    BuildSequenceDatasetFirst,
    #[default]
    HoldCurrentScope,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CoreBottleneckInputs {
    #[serde(default)]
    pub provider_auth_missing: bool,
    #[serde(default)]
    pub official_data_missing: bool,
    #[serde(default)]
    pub official_candles_missing: bool,
    #[serde(default)]
    pub outcome_links_missing: bool,
    #[serde(default)]
    pub baseline_references_missing: bool,
    #[serde(default)]
    pub no_trade_counterfactuals_missing: bool,
    #[serde(default)]
    pub risk_denied_counterfactuals_missing: bool,
    #[serde(default)]
    pub poor_calibration: bool,
    #[serde(default)]
    pub risk_overblocking: bool,
    #[serde(default)]
    pub risk_underblocking: bool,
    #[serde(default)]
    pub chair_dominated: bool,
    #[serde(default)]
    pub persona_scoring_weak: bool,
    #[serde(default)]
    pub signal_model_weak: bool,
    #[serde(default)]
    pub scenario_materialization_weak: bool,
    #[serde(default)]
    pub storage_budget_exceeded: bool,
    #[serde(default)]
    pub latency_budget_exceeded: bool,
    #[serde(default)]
    pub evidence_too_weak: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreBottleneckReport {
    pub primary_bottleneck: CoreBottleneckKind,
    pub secondary_bottlenecks: Vec<CoreBottleneckKind>,
    pub evidence: Vec<String>,
    pub recommended_next_action: CoreBottleneckRecommendation,
    pub operator_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_core_bottleneck_report(inputs: &CoreBottleneckInputs) -> CoreBottleneckReport {
    let mut candidates = Vec::new();
    let mut evidence = Vec::new();

    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.provider_auth_missing,
        CoreBottleneckKind::MissingOfficialAuth,
        "provider auth is missing for official evidence",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.official_data_missing,
        CoreBottleneckKind::MissingOfficialData,
        "non-crypto official evidence is still missing",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.official_candles_missing,
        CoreBottleneckKind::MissingOfficialCandles,
        "official candle material is missing",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.outcome_links_missing,
        CoreBottleneckKind::MissingOutcomeLinks,
        "outcome links are missing",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.baseline_references_missing,
        CoreBottleneckKind::MissingBaselineReferences,
        "baseline references are missing",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.no_trade_counterfactuals_missing,
        CoreBottleneckKind::MissingNoTradeCounterfactuals,
        "no-trade counterfactuals are missing",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.risk_denied_counterfactuals_missing,
        CoreBottleneckKind::MissingRiskDeniedCounterfactuals,
        "risk-denied counterfactuals are missing",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.risk_overblocking,
        CoreBottleneckKind::RiskOverBlocking,
        "risk governor appears to overblock soft-threshold cases",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.poor_calibration,
        CoreBottleneckKind::PoorCalibration,
        "signal calibration remains weak",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.storage_budget_exceeded,
        CoreBottleneckKind::StorageBudgetExceeded,
        "scorecard storage budget is exceeded",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.latency_budget_exceeded,
        CoreBottleneckKind::LatencyBudgetExceeded,
        "deterministic decision-path budget is exceeded",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.risk_underblocking,
        CoreBottleneckKind::RiskUnderBlocking,
        "risk governor may be underblocking",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.chair_dominated,
        CoreBottleneckKind::ChairNeedsTuning,
        "chair dominance limits committee evidence diversity",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.persona_scoring_weak,
        CoreBottleneckKind::PersonaScoringWeak,
        "persona scoring is too concentrated or weak",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.signal_model_weak,
        CoreBottleneckKind::SignalModelWeak,
        "signal edge remains weak after evidence controls",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.scenario_materialization_weak,
        CoreBottleneckKind::ScenarioMaterializationWeak,
        "scenario materialization remains thin",
    );
    push_candidate(
        &mut candidates,
        &mut evidence,
        inputs.evidence_too_weak,
        CoreBottleneckKind::EvidenceTooWeak,
        "current evidence is too weak for stronger claims",
    );

    let primary_bottleneck = candidates
        .first()
        .copied()
        .unwrap_or(CoreBottleneckKind::NoBottleneckDetected);
    let secondary_bottlenecks = candidates.iter().skip(1).copied().collect::<Vec<_>>();
    let recommended_next_action = match primary_bottleneck {
        CoreBottleneckKind::MissingOfficialAuth => {
            CoreBottleneckRecommendation::OfficialProviderAuthFirst
        }
        CoreBottleneckKind::MissingOfficialData | CoreBottleneckKind::MissingOfficialCandles => {
            CoreBottleneckRecommendation::MoreOfficialEvidence
        }
        CoreBottleneckKind::MissingOutcomeLinks => {
            CoreBottleneckRecommendation::ImproveOutcomeLinkingFirst
        }
        CoreBottleneckKind::MissingBaselineReferences => {
            CoreBottleneckRecommendation::ImproveBaselineReferenceDepth
        }
        CoreBottleneckKind::MissingNoTradeCounterfactuals
        | CoreBottleneckKind::MissingRiskDeniedCounterfactuals
        | CoreBottleneckKind::ScenarioMaterializationWeak => {
            CoreBottleneckRecommendation::ImproveCounterfactualDepthFirst
        }
        CoreBottleneckKind::RiskOverBlocking | CoreBottleneckKind::RiskUnderBlocking => {
            CoreBottleneckRecommendation::ImproveRiskGovernorFirst
        }
        CoreBottleneckKind::PoorCalibration | CoreBottleneckKind::SignalModelWeak => {
            CoreBottleneckRecommendation::ImproveSignalModelFirst
        }
        CoreBottleneckKind::ChairNeedsTuning => CoreBottleneckRecommendation::ImproveChairFirst,
        CoreBottleneckKind::PersonaScoringWeak => {
            CoreBottleneckRecommendation::ImprovePersonaScoringFirst
        }
        CoreBottleneckKind::StorageBudgetExceeded | CoreBottleneckKind::LatencyBudgetExceeded => {
            CoreBottleneckRecommendation::HoldCurrentScope
        }
        CoreBottleneckKind::EvidenceTooWeak | CoreBottleneckKind::NoBottleneckDetected => {
            CoreBottleneckRecommendation::HoldCurrentScope
        }
    };
    let operator_actions = match primary_bottleneck {
        CoreBottleneckKind::StorageBudgetExceeded | CoreBottleneckKind::LatencyBudgetExceeded => {
            vec!["compact scorecard inputs before adding scope".to_string()]
        }
        CoreBottleneckKind::MissingOfficialAuth => {
            vec!["finish provider auth and rerun official replication".to_string()]
        }
        CoreBottleneckKind::NoBottleneckDetected => {
            vec!["hold current scope until fresh official evidence arrives".to_string()]
        }
        _ => vec![format!("next_action={recommended_next_action:?}")],
    };

    let mut reason_codes = inputs.reason_codes.clone();
    reason_codes.push(ReasonCode::CoreBottleneckReportBuilt);
    if primary_bottleneck != CoreBottleneckKind::NoBottleneckDetected {
        reason_codes.push(ReasonCode::EvidenceGapDetected);
    }

    CoreBottleneckReport {
        primary_bottleneck,
        secondary_bottlenecks,
        evidence,
        recommended_next_action,
        operator_actions,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

impl CoreBottleneckReport {
    pub fn to_text(&self) -> String {
        [
            format!("primary_bottleneck={:?}", self.primary_bottleneck),
            format!(
                "secondary_bottlenecks={}",
                self.secondary_bottlenecks
                    .iter()
                    .map(|value| format!("{value:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("recommended_next_action={:?}", self.recommended_next_action),
            format!("evidence={}", self.evidence.join(" | ")),
            format!("operator_actions={}", self.operator_actions.join(" | ")),
        ]
        .join("\n")
    }
}

fn push_candidate(
    candidates: &mut Vec<CoreBottleneckKind>,
    evidence: &mut Vec<String>,
    enabled: bool,
    kind: CoreBottleneckKind,
    message: &str,
) {
    if enabled {
        candidates.push(kind);
        evidence.push(message.to_string());
    }
}
