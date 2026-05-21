use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::{
    CoreBottleneckKind, CorePerformanceFinalStatus, CorePerformanceScorecard,
    CorePerformanceScorecardBundle, CorePerformanceScorecardConfig, CorePerformanceScorecardRunner,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreScorecardRerunSummary {
    pub ran: bool,
    #[serde(default)]
    pub previous_status: Option<CorePerformanceFinalStatus>,
    #[serde(default)]
    pub current_status: Option<CorePerformanceFinalStatus>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<CoreBottleneckKind>,
    pub status_improved: bool,
    pub bottleneck_changed: bool,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoreScorecardRerun;

impl CoreScorecardRerun {
    pub fn run_bundle(&self, config_path: &str) -> Result<CorePerformanceScorecardBundle, String> {
        let config =
            CorePerformanceScorecardConfig::from_toml_path(std::path::Path::new(config_path))?;
        CorePerformanceScorecardRunner::default().run(&config)
    }

    pub fn summarize(
        &self,
        previous: Option<&CorePerformanceScorecard>,
        current: Option<&CorePerformanceScorecard>,
        mut warnings: Vec<String>,
        ran: bool,
    ) -> CoreScorecardRerunSummary {
        if !ran {
            if warnings.is_empty() {
                warnings.push("scorecard rerun was not attempted".to_string());
            }
            return CoreScorecardRerunSummary {
                ran: false,
                previous_status: previous.map(|scorecard| scorecard.final_status),
                current_status: current.map(|scorecard| scorecard.final_status),
                previous_primary_bottleneck: previous
                    .map(|scorecard| scorecard.bottleneck_report.primary_bottleneck),
                current_primary_bottleneck: current
                    .map(|scorecard| scorecard.bottleneck_report.primary_bottleneck),
                status_improved: false,
                bottleneck_changed: false,
                warnings,
                reason_codes: stable_reason_codes(&[
                    ReasonCode::CorePerformanceRegressionReportBuilt,
                ]),
            };
        }
        let previous_status = previous.map(|scorecard| scorecard.final_status);
        let current_status = current.map(|scorecard| scorecard.final_status);
        let previous_primary_bottleneck =
            previous.map(|scorecard| scorecard.bottleneck_report.primary_bottleneck);
        let current_primary_bottleneck =
            current.map(|scorecard| scorecard.bottleneck_report.primary_bottleneck);
        let bottleneck_changed = previous_primary_bottleneck != current_primary_bottleneck;
        let status_improved = matches!(
            (previous_status, current_status),
            (
                Some(CorePerformanceFinalStatus::CoreBlockedByEvidence),
                Some(CorePerformanceFinalStatus::CorePerformanceHealthyForResearch)
            ) | (
                Some(CorePerformanceFinalStatus::CoreBlockedByEvidence),
                Some(CorePerformanceFinalStatus::CoreBlockedByOutcomeLinks)
            ) | (
                Some(CorePerformanceFinalStatus::CoreBlockedByOutcomeLinks),
                Some(CorePerformanceFinalStatus::CorePerformanceHealthyForResearch)
            ) | (
                Some(CorePerformanceFinalStatus::CoreDiagnosticOnly),
                Some(CorePerformanceFinalStatus::CoreBlockedByOutcomeLinks)
            ) | (
                Some(CorePerformanceFinalStatus::CoreDiagnosticOnly),
                Some(CorePerformanceFinalStatus::CorePerformanceHealthyForResearch)
            )
        ) || matches!(
            (previous_primary_bottleneck, current_primary_bottleneck),
            (
                Some(CoreBottleneckKind::ScenarioMaterializationWeak),
                Some(CoreBottleneckKind::MissingOfficialData)
            ) | (
                Some(CoreBottleneckKind::ScenarioMaterializationWeak),
                Some(CoreBottleneckKind::SignalModelWeak)
            ) | (
                Some(CoreBottleneckKind::MissingNoTradeCounterfactuals),
                Some(CoreBottleneckKind::SignalModelWeak)
            )
        );
        CoreScorecardRerunSummary {
            ran,
            previous_status,
            current_status,
            previous_primary_bottleneck,
            current_primary_bottleneck,
            status_improved,
            bottleneck_changed,
            warnings,
            reason_codes: stable_reason_codes(&[
                ReasonCode::CorePerformanceRegressionReportBuilt,
                ReasonCode::DeterministicPath,
            ]),
        }
    }

    pub fn missing(reason: &str) -> CoreScorecardRerunSummary {
        CoreScorecardRerunSummary {
            ran: false,
            previous_status: None,
            current_status: None,
            previous_primary_bottleneck: None,
            current_primary_bottleneck: None,
            status_improved: false,
            bottleneck_changed: false,
            warnings: vec![reason.to_string()],
            reason_codes: stable_reason_codes(&[
                ReasonCode::MissingFile,
                ReasonCode::DeterministicPath,
            ]),
        }
    }
}

impl CoreScorecardRerunSummary {
    pub fn to_text(&self) -> String {
        [
            format!("ran={}", self.ran),
            format!(
                "previous_status={}",
                self.previous_status
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "current_status={}",
                self.current_status
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "previous_primary_bottleneck={}",
                self.previous_primary_bottleneck
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "current_primary_bottleneck={}",
                self.current_primary_bottleneck
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
            format!("status_improved={}", self.status_improved),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}
