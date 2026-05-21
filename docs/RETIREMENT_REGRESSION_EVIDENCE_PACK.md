# Retirement Regression Evidence Pack

Sprint 72 refines retirement evidence so a version can be summarized as ready, diagnostic-only, or still needing more evidence.

## Required evidence for a ready pack

- regression evidence
- calibration / risk context
- leaderboard comparison context
- owner rationale for retirement

## Conservative fallback

If the pack is incomplete, the result falls back to `DiagnosticOnlySupported` or `RequestMoreEvidence`. Retirement still means exclusion from comparison, not deletion.

## Main command

```bash
cargo run --quiet --bin soma_experiment -- retirement-regression-pack --config examples/soma_retirement_regression_pack.toml
```
