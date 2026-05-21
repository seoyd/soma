mod support;

use soma_zero::{RemainingBlockerQueueV4Status, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn remaining_blocker_queue_v4_keeps_family_order_explicit() {
    let config = sprint::sprint88_config_from_example(
        "soma_remaining_blocker_queue_v4.toml",
        "remaining-queue",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_remaining_blocker_queue_v4(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_remaining_blocker_queue_v4(&config)
        .expect("second");
    assert_eq!(first.primary_next_family, "CandleExpansionOps");
    assert_eq!(
        first.queue_status,
        RemainingBlockerQueueV4Status::QueueReady
    );
    assert_eq!(first.ordered_remaining_families.len(), 7);
    assert_eq!(first, second);
}
