# Candle join repair actions

Sprint 44 produces a deterministic repair plan for explainable, local-only join fixes.

## Action kinds
- `AddSymbolAlias`
- `AddTimeframeAlias`
- `AddTimestampPolicy`
- `ProvideLongerCandleWindow`
- `ProvideProvenance`
- `ProvidePreflightReport`
- `NoSafeRepairAvailable`

## Rules
- commands remain research-only
- repairs are explicit; no fuzzy mapping
- repairs cannot promote source class
- repairs cannot bypass no-lookahead safety
- missing provenance and preflight stay blocking until local artifacts exist

## Command
```bash
cargo run --bin soma_experiment -- candle-join-repair-plan --config examples/soma_candle_join_audit_symbol_mismatch.toml
```
