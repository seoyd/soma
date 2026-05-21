#[path = "support/sprint45_support.rs"]
mod sprint45_support;

use soma_zero::{BaselineBackfillSource, build_baseline_reference_backfill_plan};

#[test]
fn baseline_plan_prioritizes_existing_artifact_and_notrade_fallback() {
    let mut existing = sprint45_support::row("existing");
    existing.baseline_reference_available = false;
    let mut no_trade = sprint45_support::row("no-trade");
    no_trade.baseline_reference_available = false;
    no_trade.baseline_action = None;
    let plan = build_baseline_reference_backfill_plan("plan", &[existing, no_trade]);
    assert_eq!(plan.existing_artifact_count, 1);
    assert_eq!(plan.no_trade_fallback_count, 1);
    assert_eq!(
        plan.items[1].source,
        BaselineBackfillSource::DeterministicNoTradeBaseline
    );
}

#[test]
fn baseline_plan_marks_approximation_diagnostic_and_is_deterministic() {
    let mut approx = sprint45_support::row("approx");
    approx.baseline_reference_available = false;
    approx.baseline_action = None;
    approx.no_trade_baseline_action.clear();
    let first = build_baseline_reference_backfill_plan("plan", &[approx.clone()]);
    let second = build_baseline_reference_backfill_plan("plan", &[approx]);
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(
        first.items[0].source,
        BaselineBackfillSource::DeterministicBaselineApproximation
    );
    assert!(first.items[0].diagnostic_only);
}
