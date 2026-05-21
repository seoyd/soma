use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};
use crate::owner::{OwnerInput, OwnerReviewItem, OwnerThesisNote};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerReviewQueueSummary {
    pub pending_review_items: usize,
    pub reviewed_items: usize,
    pub deferred_items: usize,
    pub dismissed_items: usize,
    pub paper_confirmed_items: usize,
    pub blocked_items: usize,
    pub expired_items: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerPanel {
    pub review_queue_summary: OwnerReviewQueueSummary,
    #[serde(default)]
    pub pending_review_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub recent_owner_inputs: Vec<OwnerInput>,
    #[serde(default)]
    pub active_thesis_notes: Vec<OwnerThesisNote>,
    #[serde(default)]
    pub blocked_owner_inputs: Vec<OwnerInput>,
    #[serde(default)]
    pub paper_confirmed_items: Vec<OwnerReviewItem>,
    #[serde(default)]
    pub reanalysis_requests: Vec<OwnerInput>,
    #[serde(default)]
    pub owner_policy_warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OwnerPanel {
    pub fn stabilize(&mut self) {
        self.pending_review_items
            .sort_by(|left, right| left.review_id.cmp(&right.review_id));
        self.paper_confirmed_items
            .sort_by(|left, right| left.review_id.cmp(&right.review_id));
        self.recent_owner_inputs
            .sort_by(|left, right| left.owner_input_id.cmp(&right.owner_input_id));
        self.blocked_owner_inputs
            .sort_by(|left, right| left.owner_input_id.cmp(&right.owner_input_id));
        self.active_thesis_notes
            .sort_by(|left, right| left.thesis_id.cmp(&right.thesis_id));
        for item in &mut self.pending_review_items {
            item.stabilize();
        }
        for item in &mut self.paper_confirmed_items {
            item.stabilize();
        }
        for input in &mut self.recent_owner_inputs {
            input.stabilize();
        }
        for input in &mut self.blocked_owner_inputs {
            input.stabilize();
        }
        for note in &mut self.active_thesis_notes {
            note.stabilize();
        }
        self.owner_policy_warnings = stable_ordered_strings(&self.owner_policy_warnings);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}
