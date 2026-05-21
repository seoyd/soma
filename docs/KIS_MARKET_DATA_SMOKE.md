# KIS Market-Data Smoke

Sprint 58 composes auth closure, bounded dry-run, deterministic collection planning, local KIS activation, evidence depth refresh, Control Tower refresh, and operational runbook generation.

## CLI

```bash
cargo run --quiet --bin soma_experiment -- kis-market-data-smoke --config examples/soma_kis_market_data_smoke_fixture.toml
```

## Bundle contents

- auth closure report
- dry-run report
- collection plan v2
- market-data smoke report
- environment isolation report
- secret redaction audit
- Control Tower auto-refresh report
- operational runbook v2

## Constraints

- market-data-only
- local-first
- paper-only
- deterministic
- live collection disabled by default

## Notes

Smoke improvement statuses do not imply profitability, live execution readiness, or broker/account capability.
