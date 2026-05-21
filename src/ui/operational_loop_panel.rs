use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalLoopPanel {
    pub loop_status: String,
    #[serde(default)]
    pub last_loop_run_id: Option<String>,
    pub active_cycle_count: usize,
    pub generated_candidates: usize,
    pub paper_approved: usize,
    pub paper_open: usize,
    pub risk_blocked: usize,
    pub no_trade: usize,
    pub owner_review_pending: usize,
    pub next_action: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TrinityStatusView {
    pub persona_id: String,
    pub status: String,
    #[serde(default)]
    pub current_candidate_id: Option<String>,
    #[serde(default)]
    pub current_symbol: Option<String>,
    #[serde(default)]
    pub last_stance: Option<String>,
    #[serde(default)]
    pub last_conviction: Option<f64>,
    #[serde(default)]
    pub last_voice_power: Option<f64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TrinityStatusPanel {
    #[serde(default)]
    pub persona_views: Vec<TrinityStatusView>,
    pub active_count: usize,
    pub idle_count: usize,
    pub analyzing_count: usize,
    pub voting_count: usize,
    pub blocked_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaperLifecyclePanel {
    pub open_positions: usize,
    pub closed_positions: usize,
    pub target_hit_count: usize,
    pub stop_hit_count: usize,
    pub expired_count: usize,
    pub risk_closed_count: usize,
    #[serde(default)]
    pub average_unrealized_return: Option<f64>,
    #[serde(default)]
    pub average_realized_return: Option<f64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OperationalLoopPanel {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl TrinityStatusPanel {
    pub fn stabilize(&mut self) {
        self.persona_views
            .sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
        for view in &mut self.persona_views {
            view.reason_codes = stable_reason_codes(&view.reason_codes);
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

impl PaperLifecyclePanel {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}
