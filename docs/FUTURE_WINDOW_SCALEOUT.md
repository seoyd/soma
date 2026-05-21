# Future Window Scaleout

Sprint 47 groups future-window gaps by symbol/timeframe and emits bounded local reuse or extension plans.

## Behavior
- groups rows deterministically
- prefers local extension when enabled
- provider jobs are planned only when explicitly enabled
- never runs live collection from this planner
- plans expose `scaleout_id`, grouped requirements, bounded jobs, runnable/skipped counts, operator actions, and a storage budget summary

## CLI
`cargo run --bin soma_experiment -- future-window-scaleout-plan --config examples/soma_future_window_scaleout_multi_row.toml`
