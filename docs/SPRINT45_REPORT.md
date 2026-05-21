# Sprint 45 Report

## Implemented items
- official-ready row inventory config, report, and runner
- scenario materialization v3 config, report, and runner
- outcome, baseline, and counterfactual backfill plans
- complete comparable row builder, closure runner, storage bundle, and bottleneck movement report
- Sprint 45 CLI wiring, examples, docs, and tests

## Validation
- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- CLI smoke:
  - `official-ready-row-inventory --config examples/soma_official_ready_row_inventory_official_replication.toml`
  - `scenario-materialize-v3 --config examples/soma_scenario_materialize_v3_official_replication.toml`
  - `complete-row-close --config examples/soma_complete_row_close_official_replication.toml`
  - `complete-row-close --config examples/soma_complete_row_close_controlled.toml`
  - `complete-row-close --config examples/soma_complete_row_close_crypto_only.toml`
  - `complete-row-close --config examples/soma_complete_row_close_diagnostics_only.toml`

## Inventory status
Official-ready matches are now reported separately from complete comparable rows, with deterministic missing-reference counts and source-boundary flags.

## Scenario materialization v3 status
Materialization prefers existing row-level scenarios, then bounded local projection, while keeping limited-feature and summary-derived outputs explicitly weaker.

## Complete row closure status
Closure now reports whether complete rows improved, whether only partial closure happened, and whether the remaining blocker is scenario, outcome, baseline, or counterfactual depth.

## Core bottleneck movement
Bottleneck movement is reported conservatively and supports non-scenario bottleneck transitions without recursive failure.

## Risk review
- no live trading path added
- no broker, order, or account path added
- no runtime LLM path added
- no Mamba runtime added
- no source promotion through backfill
- no hidden lookahead introduced

## Next sprint recommendation
Use Sprint 46 to deepen official outcome linkage and counterfactual depth on the rows that Sprint 45 can now inventory and close deterministically.
