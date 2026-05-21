use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OperationalEventKind {
    #[default]
    CandidateGenerated,
    CandidateStateChanged,
    PersonaStartedAnalysis,
    PersonaVoted,
    ChairReviewed,
    RiskReviewed,
    OwnerReviewQueued,
    OwnerInputApplied,
    PaperApproved,
    PaperPositionOpened,
    PaperPositionClosed,
    NoTrade,
    RiskBlocked,
    ReanalysisRequested,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalAuditEvent {
    pub event_id: String,
    pub event_kind: OperationalEventKind,
    #[serde(default)]
    pub candidate_id: Option<String>,
    #[serde(default)]
    pub persona_id: Option<String>,
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
    #[serde(default)]
    pub status_before: Option<String>,
    #[serde(default)]
    pub status_after: Option<String>,
    pub summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalAuditTimeline {
    #[serde(default)]
    pub events: Vec<OperationalAuditEvent>,
    pub event_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub fingerprint: String,
}

impl OperationalAuditEvent {
    pub fn new(
        event_kind: OperationalEventKind,
        candidate_id: Option<String>,
        persona_id: Option<String>,
        timestamp_ms: Option<u64>,
        status_before: Option<String>,
        status_after: Option<String>,
        summary: impl Into<String>,
        reason_codes: Vec<ReasonCode>,
    ) -> Self {
        let summary = summary.into();
        Self {
            event_id: stable_hash_string(&format!(
                "{:?}|{:?}|{:?}|{:?}|{:?}|{}",
                event_kind, candidate_id, persona_id, timestamp_ms, status_after, summary
            )),
            event_kind,
            candidate_id,
            persona_id,
            timestamp_ms,
            status_before,
            status_after,
            summary,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }
}

impl OperationalAuditTimeline {
    pub fn from_events(mut events: Vec<OperationalAuditEvent>) -> Self {
        events.sort_by(|left, right| {
            left.timestamp_ms
                .unwrap_or_default()
                .cmp(&right.timestamp_ms.unwrap_or_default())
                .then(left.event_id.cmp(&right.event_id))
        });
        for event in &mut events {
            event.reason_codes = stable_reason_codes(&event.reason_codes);
        }
        let warning_count = events
            .iter()
            .filter(|event| {
                event.event_kind != OperationalEventKind::Error && event.reason_codes.len() > 1
            })
            .count();
        let error_count = events
            .iter()
            .filter(|event| matches!(event.event_kind, OperationalEventKind::Error))
            .count();
        let mut timeline = Self {
            event_count: events.len(),
            warning_count,
            error_count,
            events,
            fingerprint: String::new(),
        };
        timeline.stabilize();
        timeline
    }

    pub fn stabilize(&mut self) {
        self.events.sort_by(|left, right| {
            left.timestamp_ms
                .unwrap_or_default()
                .cmp(&right.timestamp_ms.unwrap_or_default())
                .then(left.event_id.cmp(&right.event_id))
        });
        for event in &mut self.events {
            event.reason_codes = stable_reason_codes(&event.reason_codes);
        }
        self.event_count = self.events.len();
        self.warning_count = self
            .events
            .iter()
            .filter(|event| {
                event.event_kind != OperationalEventKind::Error && event.reason_codes.len() > 1
            })
            .count();
        self.error_count = self
            .events
            .iter()
            .filter(|event| matches!(event.event_kind, OperationalEventKind::Error))
            .count();
        self.fingerprint = String::new();
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "audit_warning=operational audit timeline is deterministic local audit state only"
                .to_string(),
            format!("event_count={}", self.event_count),
            format!("warning_count={}", self.warning_count),
            format!("error_count={}", self.error_count),
            format!("fingerprint={}", self.fingerprint),
        ];
        lines.extend(self.events.iter().map(|event| {
            format!(
                "event={:?};candidate_id={};persona_id={};summary={}",
                event.event_kind,
                event.candidate_id.clone().unwrap_or_default(),
                event.persona_id.clone().unwrap_or_default(),
                event.summary
            )
        }));
        lines.join("\n")
    }
}
