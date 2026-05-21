# Official row injection

`official-row-inject` materializes committee scenario rows in priority order:

1. existing official committee pack
2. evidence-lane rows
3. canonical CSV fallback rows

## Command

```bash
cargo run --bin soma_experiment -- official-row-inject --config examples/soma_official_row_inject.toml
```

## Rules

- row-level official provenance is required by default
- preflight readiness is required by default
- summary-derived rows are skipped unless explicitly allowed
- yfinance and fixtures stay separated from official evidence
- crypto-only official rows remain crypto-only
