use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerInputKind {
    WatchlistAdd,
    WatchlistRemove,
    CandidateNote,
    CandidateHold,
    CandidateDismiss,
    CandidateReanalysisRequest,
    PaperConfirm,
    MarkReviewed,
    ThesisNote,
    RiskTightenRequest,
    RiskLoosenRequestDiagnosticOnly,
    StrategyPreference,
    ProviderPreference,
    DataRequest,
    EvidenceRequest,
    Abstain,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerInputStatus {
    #[default]
    Draft,
    Submitted,
    Accepted,
    Rejected,
    Applied,
    Ignored,
    BlockedByRiskGovernor,
    BlockedByPolicy,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerInputTargetType {
    Candidate,
    Symbol,
    Market,
    Provider,
    CommitteeDecision,
    RiskDecision,
    EvidenceRun,
    #[default]
    System,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerInput {
    pub owner_input_id: String,
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
    #[serde(default)]
    pub owner_id: Option<String>,
    pub input_kind: OwnerInputKind,
    pub target_type: OwnerInputTargetType,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(default)]
    pub freeform_note: Option<String>,
    #[serde(default)]
    pub structured_payload: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub requested_action: Option<String>,
    pub status: OwnerInputStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for OwnerInput {
    fn default() -> Self {
        Self {
            owner_input_id: "owner-input-draft".to_string(),
            timestamp_ms: None,
            owner_id: None,
            input_kind: OwnerInputKind::Unknown,
            target_type: OwnerInputTargetType::System,
            target_id: None,
            symbol: None,
            market: None,
            freeform_note: None,
            structured_payload: None,
            requested_action: None,
            status: OwnerInputStatus::Draft,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OwnerInput {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn with_fingerprint(mut self) -> Self {
        self.stabilize();
        self
    }

    pub fn fingerprint(&self) -> String {
        let mut copy = self.clone();
        copy.stabilize();
        stable_hash_string(
            &serde_json::to_string(&copy).unwrap_or_else(|_| copy.owner_input_id.clone()),
        )
    }

    pub fn freeform_only(&self) -> bool {
        self.freeform_note.is_some()
            && self
                .structured_payload
                .as_ref()
                .is_none_or(|payload| payload.is_empty())
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self.input_kind, OwnerInputKind::Unknown)
    }

    pub fn is_diagnostic_only_kind(&self) -> bool {
        matches!(
            self.input_kind,
            OwnerInputKind::RiskLoosenRequestDiagnosticOnly
                | OwnerInputKind::StrategyPreference
                | OwnerInputKind::ProviderPreference
                | OwnerInputKind::DataRequest
                | OwnerInputKind::EvidenceRequest
                | OwnerInputKind::Abstain
        )
    }

    pub fn requests_forbidden_runtime_action(&self) -> bool {
        let requested = self
            .requested_action
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let freeform = self
            .freeform_note
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        [requested, freeform].into_iter().any(|text| {
            [
                "live trade",
                "live trading",
                "execute order",
                "place trade",
                "broker",
                "account",
                "balance",
                "holding",
                "position",
                "override risk",
                "loosen hard veto",
                "buying power",
                "cancel order",
                "execution",
                "kis order",
            ]
            .iter()
            .any(|needle| text.contains(needle))
        })
    }

    pub fn summary_line(&self) -> String {
        format!(
            "owner_input_id={}\ninput_kind={:?}\ntarget_type={:?}\ntarget_id={}\nstatus={:?}\nfingerprint={}",
            self.owner_input_id,
            self.input_kind,
            self.target_type,
            self.target_id.clone().unwrap_or_default(),
            self.status,
            self.fingerprint()
        )
    }
}
