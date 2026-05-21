use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::CoreBottleneckKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoreBottleneckMovementKind {
    #[default]
    NoMovement,
    MovedFromScenarioMaterializationWeak,
    MovedToOutcomeLinking,
    MovedToBaselineReferenceDepth,
    MovedToCounterfactualDepth,
    MovedToRiskGovernor,
    MovedToSignalQuality,
    MovedToOfficialEvidence,
    MovedToBudget,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreBottleneckMovementReport {
    #[serde(default)]
    pub previous_primary_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<CoreBottleneckKind>,
    pub bottleneck_changed: bool,
    pub movement_kind: CoreBottleneckMovementKind,
    pub interpretation: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_core_bottleneck_movement_report(
    previous_primary_bottleneck: Option<CoreBottleneckKind>,
    current_primary_bottleneck: Option<CoreBottleneckKind>,
) -> CoreBottleneckMovementReport {
    let movement_kind = determine_movement(previous_primary_bottleneck, current_primary_bottleneck);
    let bottleneck_changed = movement_kind != CoreBottleneckMovementKind::NoMovement;
    let interpretation = match movement_kind {
        CoreBottleneckMovementKind::NoMovement => "primary core bottleneck did not move; closure remains research-only and needs more evidence".to_string(),
        CoreBottleneckMovementKind::MovedFromScenarioMaterializationWeak => "scenario materialization is no longer the primary blocker; continue with the newly surfaced bottleneck conservatively".to_string(),
        CoreBottleneckMovementKind::MovedToOutcomeLinking => "materialization improved enough to expose outcome linkage gaps; no profitability claim is implied".to_string(),
        CoreBottleneckMovementKind::MovedToBaselineReferenceDepth => "baseline depth is now the next bottleneck after materialization improvements".to_string(),
        CoreBottleneckMovementKind::MovedToCounterfactualDepth => "counterfactual depth is now the next bottleneck after materialization improvements".to_string(),
        CoreBottleneckMovementKind::MovedToRiskGovernor => "risk-governor behavior is now the next bottleneck; defensive semantics remain final".to_string(),
        CoreBottleneckMovementKind::MovedToSignalQuality => "signal quality or calibration weaknesses are now more prominent than scenario materialization".to_string(),
        CoreBottleneckMovementKind::MovedToOfficialEvidence => "official evidence depth remains the next bottleneck even after closure work".to_string(),
        CoreBottleneckMovementKind::MovedToBudget => "storage or latency budget pressure is now the next bottleneck; no live-readiness claim is implied".to_string(),
    };
    CoreBottleneckMovementReport {
        previous_primary_bottleneck,
        current_primary_bottleneck,
        bottleneck_changed,
        movement_kind,
        interpretation,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CoreBottleneckReportBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl CoreBottleneckMovementReport {
    pub fn to_text(&self) -> String {
        [
            format!(
                "previous_primary_bottleneck={}",
                self.previous_primary_bottleneck
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "current_primary_bottleneck={}",
                self.current_primary_bottleneck
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("movement_kind={:?}", self.movement_kind),
            format!("interpretation={}", self.interpretation),
        ]
        .join("\n")
    }
}

fn determine_movement(
    previous: Option<CoreBottleneckKind>,
    current: Option<CoreBottleneckKind>,
) -> CoreBottleneckMovementKind {
    if previous == current {
        return CoreBottleneckMovementKind::NoMovement;
    }
    if previous == Some(CoreBottleneckKind::ScenarioMaterializationWeak)
        && current != Some(CoreBottleneckKind::ScenarioMaterializationWeak)
    {
        return movement_from_current(current)
            .unwrap_or(CoreBottleneckMovementKind::MovedFromScenarioMaterializationWeak);
    }
    movement_from_current(current).unwrap_or(CoreBottleneckMovementKind::NoMovement)
}

fn movement_from_current(
    current: Option<CoreBottleneckKind>,
) -> Option<CoreBottleneckMovementKind> {
    match current {
        Some(CoreBottleneckKind::MissingOutcomeLinks) => {
            Some(CoreBottleneckMovementKind::MovedToOutcomeLinking)
        }
        Some(CoreBottleneckKind::MissingBaselineReferences) => {
            Some(CoreBottleneckMovementKind::MovedToBaselineReferenceDepth)
        }
        Some(CoreBottleneckKind::MissingNoTradeCounterfactuals)
        | Some(CoreBottleneckKind::MissingRiskDeniedCounterfactuals) => {
            Some(CoreBottleneckMovementKind::MovedToCounterfactualDepth)
        }
        Some(CoreBottleneckKind::RiskOverBlocking)
        | Some(CoreBottleneckKind::RiskUnderBlocking) => {
            Some(CoreBottleneckMovementKind::MovedToRiskGovernor)
        }
        Some(CoreBottleneckKind::PoorCalibration)
        | Some(CoreBottleneckKind::ChairNeedsTuning)
        | Some(CoreBottleneckKind::PersonaScoringWeak)
        | Some(CoreBottleneckKind::SignalModelWeak) => {
            Some(CoreBottleneckMovementKind::MovedToSignalQuality)
        }
        Some(CoreBottleneckKind::MissingOfficialAuth)
        | Some(CoreBottleneckKind::MissingOfficialData)
        | Some(CoreBottleneckKind::MissingOfficialCandles)
        | Some(CoreBottleneckKind::EvidenceTooWeak) => {
            Some(CoreBottleneckMovementKind::MovedToOfficialEvidence)
        }
        Some(CoreBottleneckKind::StorageBudgetExceeded)
        | Some(CoreBottleneckKind::LatencyBudgetExceeded) => {
            Some(CoreBottleneckMovementKind::MovedToBudget)
        }
        _ => None,
    }
}
