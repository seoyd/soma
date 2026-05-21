use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::{CoreBottleneckKind, CorePerformanceFinalStatus};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorePerformanceRerunAfterOutcomeLinkage {
    pub ran: bool,
    #[serde(default)]
    pub previous_status: Option<CorePerformanceFinalStatus>,
    #[serde(default)]
    pub current_status: Option<CorePerformanceFinalStatus>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<CoreBottleneckKind>,
    pub bottleneck_changed: bool,
    pub bottleneck_moved_from_evidence_too_weak: bool,
    pub status_improved: bool,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CorePerformanceRerunAfterOutcomeLinkage {
    pub fn build(
        previous_primary_bottleneck: Option<CoreBottleneckKind>,
        current_primary_bottleneck: Option<CoreBottleneckKind>,
        previous_status: Option<CorePerformanceFinalStatus>,
        current_status: Option<CorePerformanceFinalStatus>,
        ran: bool,
        mut warnings: Vec<String>,
    ) -> Self {
        if !ran && warnings.is_empty() {
            warnings.push("core performance rerun was not possible".to_string());
        }
        let bottleneck_changed = previous_primary_bottleneck != current_primary_bottleneck;
        let bottleneck_moved_from_evidence_too_weak = previous_primary_bottleneck
            == Some(CoreBottleneckKind::EvidenceTooWeak)
            && current_primary_bottleneck != Some(CoreBottleneckKind::EvidenceTooWeak);
        let status_improved = status_rank(current_status) > status_rank(previous_status)
            || bottleneck_moved_from_evidence_too_weak;
        Self {
            ran,
            previous_status,
            current_status,
            previous_primary_bottleneck,
            current_primary_bottleneck,
            bottleneck_changed,
            bottleneck_moved_from_evidence_too_weak,
            status_improved,
            warnings,
            reason_codes: stable_reason_codes(&[
                ReasonCode::CorePerformanceRegressionReportBuilt,
                ReasonCode::DeterministicPath,
            ]),
        }
    }

    pub fn missing(reason: &str) -> Self {
        Self::build(None, None, None, None, false, vec![reason.to_string()])
    }

    pub fn to_text(&self) -> String {
        [
            format!("ran={}", self.ran),
            format!(
                "previous_status={}",
                self.previous_status
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "current_status={}",
                self.current_status
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
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
            format!(
                "bottleneck_moved_from_evidence_too_weak={}",
                self.bottleneck_moved_from_evidence_too_weak
            ),
            format!("status_improved={}", self.status_improved),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

fn status_rank(status: Option<CorePerformanceFinalStatus>) -> i32 {
    match status.unwrap_or(CorePerformanceFinalStatus::CoreDiagnosticOnly) {
        CorePerformanceFinalStatus::CoreDiagnosticOnly => 0,
        CorePerformanceFinalStatus::CoreNeedsMoreEvidence => 1,
        CorePerformanceFinalStatus::CoreBlockedByEvidence => 2,
        CorePerformanceFinalStatus::CoreBlockedByOfficialData => 3,
        CorePerformanceFinalStatus::CoreBlockedByOutcomeLinks => 4,
        CorePerformanceFinalStatus::CoreBlockedByRiskBehavior => 5,
        CorePerformanceFinalStatus::CoreBlockedByCalibration => 6,
        CorePerformanceFinalStatus::CoreBlockedByBudget => 7,
        CorePerformanceFinalStatus::CorePerformanceHealthyForResearch => 8,
    }
}
