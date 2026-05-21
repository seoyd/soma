use crate::core::{Side, TradeProposal};

pub fn risk_reward_ratio(proposal: &TradeProposal) -> Option<f64> {
    let stop = proposal.stop_loss?;
    let take = proposal.take_profit?;
    match proposal.side {
        Side::Long => {
            let risk = proposal.entry_price_hint - stop;
            let reward = take - proposal.entry_price_hint;
            if risk > 0.0 && reward > 0.0 {
                Some(reward / risk)
            } else {
                None
            }
        }
        Side::Short => {
            let risk = stop - proposal.entry_price_hint;
            let reward = proposal.entry_price_hint - take;
            if risk > 0.0 && reward > 0.0 {
                Some(reward / risk)
            } else {
                None
            }
        }
    }
}

pub fn projected_total_exposure(current_exposure: f64, quantity_hint: f64) -> f64 {
    current_exposure + quantity_hint.max(0.0)
}
