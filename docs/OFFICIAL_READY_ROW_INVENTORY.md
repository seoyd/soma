# Official Ready Row Inventory

Sprint 45 separates **official-ready candle match** from a **complete comparable row**.

## Why the split matters
- an official-ready match only says the row can be aligned to bounded local candle evidence
- a complete comparable row still needs scenario materialization, outcome reference, baseline reference, NoTrade counterfactual, RiskDenied counterfactual, and `no_lookahead_safe = true`
- therefore official-ready counts are not usefulness, profitability, or live-readiness claims

## Missing-reference taxonomy
The inventory reports deterministic gap counts for:
- missing scenario row
- missing outcome reference
- missing baseline reference
- missing NoTrade counterfactual
- missing RiskDenied counterfactual
- summary-derived only rows
- source-ineligible rows
- no-lookahead violations

## Boundary rules
- controlled evidence stays diagnostic-only
- crypto-only evidence stays crypto-only
- yfinance stays research-only
- fixture evidence stays architecture-test-only
- backfill cannot promote source class

## Command
```bash
cargo run --bin soma_experiment -- official-ready-row-inventory --config examples/soma_official_ready_row_inventory_official_replication.toml
```
