use crate::core::Stance;

#[derive(Clone, Debug, PartialEq)]
pub struct ShadowVoteRecord {
    pub persona_id: String,
    pub selected_for_decision: bool,
    pub affected_live_decision: bool,
    pub hypothetical_stance: Stance,
    pub evaluation_pending: bool,
}
