use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardEventKind {
    #[default]
    ProviderStatus,
    DataCollection,
    Preflight,
    EvidenceUpdate,
    ScenarioMaterialization,
    CommitteeVote,
    ChairDecision,
    RiskDecision,
    CandidateStateChange,
    PaperPositionStateChange,
    HumanConfirmStateChange,
    OwnerInputSubmitted,
    OwnerInputApplied,
    OwnerInputBlocked,
    HumanConfirmTransition,
    CandidateHeld,
    CandidateDismissed,
    PaperConfirmed,
    ReanalysisRequested,
    BottleneckChange,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardEventSeverity {
    #[default]
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub event_id: String,
    pub kind: DashboardEventKind,
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub source_report: Option<String>,
    pub severity: DashboardEventSeverity,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTimelinePanel {
    #[serde(default)]
    pub events: Vec<DashboardEvent>,
    pub warnings: usize,
    pub errors: usize,
    pub critical_count: usize,
    pub fingerprint: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl AuditTimelinePanel {
    pub fn stabilize(&mut self) {
        self.events.sort_by(|left, right| {
            left.timestamp_ms
                .cmp(&right.timestamp_ms)
                .then_with(|| format!("{:?}", left.severity).cmp(&format!("{:?}", right.severity)))
                .then_with(|| left.event_id.cmp(&right.event_id))
        });
        for event in &mut self.events {
            event.reason_codes = stable_reason_codes(&event.reason_codes);
        }
        self.warnings = self
            .events
            .iter()
            .filter(|event| matches!(event.severity, DashboardEventSeverity::Warning))
            .count();
        self.errors = self
            .events
            .iter()
            .filter(|event| matches!(event.severity, DashboardEventSeverity::Error))
            .count();
        self.critical_count = self
            .events
            .iter()
            .filter(|event| matches!(event.severity, DashboardEventSeverity::Critical))
            .count();
        self.fingerprint = stable_hash_string(
            &self
                .events
                .iter()
                .map(|event| {
                    format!(
                        "{}|{:?}|{}|{}|{}|{:?}",
                        event.event_id,
                        event.kind,
                        event.timestamp_ms.unwrap_or_default(),
                        event.title,
                        event.summary,
                        event.severity,
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}
