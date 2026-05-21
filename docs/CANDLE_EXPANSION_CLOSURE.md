# Candle Expansion Closure

Sprint 43 closes the loop from explicit candle gaps to bounded expansion output.

## Before / after interpretation
1. load the current gap map
2. build the bounded acquisition plan
3. run local import jobs first, and collection jobs only when enabled
4. rebuild candle coverage from the expanded pack
5. optionally rerun comparable backfill and scorecard summaries
6. compare the before/after gap counts and bottlenecks conservatively

## Closure rules
- official improvement means more official-ready coverage, not live readiness.
- diagnostic-only or research-only sources stay diagnostic-only after the run.
- bottleneck movement is recorded, but never overclaimed.
- scorecard reruns remain research-only evidence.

## Commands
```bash
cargo run --bin soma_experiment -- candle-expand --config examples/soma_candle_expand_official_replication.toml
cargo run --bin soma_experiment -- candle-expand --config examples/soma_candle_expand_controlled.toml
cargo run --bin soma_experiment -- candle-expand --config examples/soma_candle_expand_diagnostics_only.toml
```
