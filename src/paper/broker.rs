use crate::core::{OrderPlan, PaperOrder, PaperOrderStatus, ReasonCode, Side};

use super::Ledger;

pub trait Broker {
    fn supports_live_execution(&self) -> bool;
    fn live_call_count(&self) -> usize;
    fn submit_paper_order(
        &mut self,
        plan: OrderPlan,
        timestamp_ms: u64,
        reason_codes: Vec<ReasonCode>,
    ) -> PaperOrder;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaperBroker {
    pub ledger: Ledger,
    live_call_count: usize,
}

impl Broker for PaperBroker {
    fn supports_live_execution(&self) -> bool {
        false
    }

    fn live_call_count(&self) -> usize {
        self.live_call_count
    }

    fn submit_paper_order(
        &mut self,
        plan: OrderPlan,
        timestamp_ms: u64,
        reason_codes: Vec<ReasonCode>,
    ) -> PaperOrder {
        let order = PaperOrder {
            order_id: self.ledger.next_order_id(),
            symbol: plan.symbol,
            side: match plan.side {
                Side::Long => Side::Long,
                Side::Short => Side::Short,
            },
            quantity: plan.quantity,
            entry_price: plan.entry_price,
            stop_loss: plan.stop_loss,
            take_profit: plan.take_profit,
            paper_only: true,
            status: PaperOrderStatus::Accepted,
            timestamp_ms,
            reason_codes,
        };
        self.ledger.record_order(order.clone());
        order
    }
}
