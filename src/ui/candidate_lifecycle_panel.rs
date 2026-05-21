use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateLifecycleView {
    pub candidate_id: String,
    pub symbol: String,
    pub market: String,
    pub source_kind: String,
    pub evidence_class: String,
    pub lifecycle_status: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateLifecyclePanel {
    #[serde(default)]
    pub candidate_views: Vec<CandidateLifecycleView>,
    #[serde(default)]
    pub status_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandidateLifecyclePanel {
    pub fn stabilize(&mut self) {
        self.candidate_views
            .sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        self.status_counts.clear();
        for view in &mut self.candidate_views {
            view.reason_codes = stable_reason_codes(&view.reason_codes);
            *self
                .status_counts
                .entry(view.lifecycle_status.clone())
                .or_insert(0) += 1;
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}
