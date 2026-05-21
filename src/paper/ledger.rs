use crate::core::{AuditEvent, PaperOrder};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ledger {
    pub orders: Vec<PaperOrder>,
    pub audits: Vec<AuditEvent>,
    next_order_number: u64,
}

impl Ledger {
    pub fn next_order_id(&mut self) -> String {
        self.next_order_number += 1;
        format!("paper-{:06}", self.next_order_number)
    }

    pub fn record_order(&mut self, order: PaperOrder) {
        self.orders.push(order);
    }

    pub fn record_audit(&mut self, audit: AuditEvent) {
        self.audits.push(audit);
    }
}
