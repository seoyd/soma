mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn paper_entry_timing_run_counts_each_window() {
    let bundle = run_sprint102("soma_paper_entry_timing_run.toml", "sprint102-entry-timing");
    let run = &bundle.paper_only_entry_timing_proposal_run;
    assert!(run.next_candle_count > 0);
    assert!(run.next_n_candles_count > 0);
    assert!(run.pullback_confirmation_count > 0);
    assert!(run.breakout_retest_count > 0);
    assert!(run.volatility_cooldown_count > 0);
    assert!(run.no_entry_count > 0);
    assert_eq!(run, &run.clone());
}
