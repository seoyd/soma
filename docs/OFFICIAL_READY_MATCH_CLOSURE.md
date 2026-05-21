# Official-ready match closure

Sprint 44 adds a bounded closure runner that audits joins, applies only safe explicit repairs, reruns local summaries, and reports whether the bottleneck moved.

## Flow
1. run join audit
2. build deterministic repair plan
3. apply safe local repairs only
4. rerun join audit
5. optionally rerun backfill / reference / counterfactual / scorecard summaries
6. emit closure bundle under `target/soma_official_ready_match_closure/<closure_id>/`

## Closure rules
- official-ready improvement does not imply live readiness or profitability
- source class is never promoted
- no-lookahead unsafe matches are rejected
- controlled evidence stays diagnostic-only
- yfinance stays research-only
- fixture evidence stays architecture-test-only
- crypto-only evidence stays crypto-only

## Commands
```bash
cargo run --bin soma_experiment -- official-ready-match-close --config examples/soma_official_ready_match_close_official_replication.toml
cargo run --bin soma_experiment -- official-ready-match-close --config examples/soma_official_ready_match_close_controlled.toml
cargo run --bin soma_experiment -- official-ready-match-close --config examples/soma_official_ready_match_close_diagnostics_only.toml
```
