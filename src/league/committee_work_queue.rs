use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::owner::{OwnerInput, OwnerInputKind};

use super::candidate_generation::GeneratedCandidate;
use super::candidate_lifecycle::CandidateLifecycleStatus;
use super::trinity_personas::active_trinity_scorers;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeTaskKind {
    #[default]
    AnalyzeCandidate,
    VoteCandidate,
    ReanalyzeCandidate,
    ReviewRiskBlocked,
    ReviewOwnerHeld,
    MonitorPaperPosition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeTaskStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Blocked,
    Skipped,
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeWorkItem {
    pub work_id: String,
    pub candidate_id: String,
    pub task_kind: CommitteeTaskKind,
    #[serde(default)]
    pub assigned_personas: Vec<String>,
    pub status: CommitteeTaskStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeWorkQueue {
    #[serde(default)]
    pub pending_items: Vec<CommitteeWorkItem>,
    #[serde(default)]
    pub running_items: Vec<CommitteeWorkItem>,
    #[serde(default)]
    pub completed_items: Vec<CommitteeWorkItem>,
    #[serde(default)]
    pub blocked_items: Vec<CommitteeWorkItem>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

impl CommitteeWorkQueue {
    pub fn stabilize(&mut self) {
        for items in [
            &mut self.pending_items,
            &mut self.running_items,
            &mut self.completed_items,
            &mut self.blocked_items,
        ] {
            items.sort_by(|left, right| left.work_id.cmp(&right.work_id));
            for item in items.iter_mut() {
                item.assigned_personas.sort();
                item.assigned_personas.dedup();
                item.reason_codes = stable_reason_codes(&item.reason_codes);
            }
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }
}

pub fn build_committee_work_queue(
    candidates: &[GeneratedCandidate],
    owner_inputs: &[OwnerInput],
) -> CommitteeWorkQueue {
    let assignable_personas = active_trinity_scorers()
        .into_iter()
        .map(|scorer| scorer.card().persona_id)
        .collect::<Vec<_>>();
    let mut queue = CommitteeWorkQueue {
        pending_items: Vec::new(),
        running_items: Vec::new(),
        completed_items: Vec::new(),
        blocked_items: Vec::new(),
        reason_codes: vec![ReasonCode::DeterministicPath],
        fingerprint: String::new(),
    };

    let mut ordered_candidates = candidates.to_vec();
    ordered_candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    for candidate in ordered_candidates {
        let base_status = match candidate.initial_status {
            CandidateLifecycleStatus::DiagnosticOnly => CommitteeTaskStatus::DiagnosticOnly,
            CandidateLifecycleStatus::RiskBlocked => CommitteeTaskStatus::Blocked,
            _ => CommitteeTaskStatus::Pending,
        };
        if matches!(
            candidate.initial_status,
            CandidateLifecycleStatus::EvidenceReady | CandidateLifecycleStatus::Detected
        ) {
            queue.pending_items.push(work_item(
                &candidate,
                CommitteeTaskKind::AnalyzeCandidate,
                base_status,
                &assignable_personas,
                Vec::new(),
            ));
            queue.pending_items.push(work_item(
                &candidate,
                CommitteeTaskKind::VoteCandidate,
                base_status,
                &assignable_personas,
                Vec::new(),
            ));
        }
        if owner_inputs.iter().any(|input| {
            matches!(input.input_kind, OwnerInputKind::CandidateReanalysisRequest)
                && (input.target_id.as_deref() == Some(candidate.candidate_id.as_str())
                    || input.symbol.as_deref() == Some(candidate.symbol.as_str()))
        }) {
            queue.pending_items.push(work_item(
                &candidate,
                CommitteeTaskKind::ReanalyzeCandidate,
                CommitteeTaskStatus::Pending,
                &assignable_personas,
                Vec::new(),
            ));
        }
        if matches!(
            candidate.initial_status,
            CandidateLifecycleStatus::RiskBlocked
        ) {
            queue.blocked_items.push(work_item(
                &candidate,
                CommitteeTaskKind::ReviewRiskBlocked,
                CommitteeTaskStatus::Blocked,
                &[],
                vec!["risk governor denied current cycle".to_string()],
            ));
        }
        if matches!(
            candidate.initial_status,
            CandidateLifecycleStatus::OwnerHeld
        ) {
            queue.pending_items.push(work_item(
                &candidate,
                CommitteeTaskKind::ReviewOwnerHeld,
                CommitteeTaskStatus::Pending,
                &[],
                Vec::new(),
            ));
        }
        if matches!(
            candidate.initial_status,
            CandidateLifecycleStatus::PaperApproved
                | CandidateLifecycleStatus::PaperPositionOpen
                | CandidateLifecycleStatus::PaperPositionClosed
        ) {
            queue.pending_items.push(work_item(
                &candidate,
                CommitteeTaskKind::MonitorPaperPosition,
                CommitteeTaskStatus::Pending,
                &[],
                Vec::new(),
            ));
        }
    }

    queue.stabilize();
    queue
}

fn work_item(
    candidate: &GeneratedCandidate,
    task_kind: CommitteeTaskKind,
    status: CommitteeTaskStatus,
    assigned_personas: &[String],
    blockers: Vec<String>,
) -> CommitteeWorkItem {
    CommitteeWorkItem {
        work_id: stable_hash_string(&format!(
            "{}|{:?}|{:?}",
            candidate.candidate_id, task_kind, status
        )),
        candidate_id: candidate.candidate_id.clone(),
        task_kind,
        assigned_personas: assigned_personas.to_vec(),
        status,
        blockers,
        reason_codes: stable_reason_codes(&[ReasonCode::DeterministicPath]),
    }
}
