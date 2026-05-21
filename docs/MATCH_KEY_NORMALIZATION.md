# Match-key normalization

Sprint 44 normalizes row-to-candle join keys without weakening source boundaries.

## Preserved fields
- raw symbol
- provider symbol
- normalized symbol
- venue
- market
- timeframe
- timestamp policy
- source class

## Rules
- alias maps are explicit TOML files and must be local
- timeframe remaps are explicit and deterministic
- timestamp/session policy remaps are explicit and reason-coded
- yfinance, fixture, controlled, and crypto-only rows keep their original source constraints
- normalization never fabricates official readiness

## Example maps
- `examples/sprint44_data/symbol_alias_map.toml`
- `examples/sprint44_data/timeframe_alias_map.toml`
- `examples/sprint44_data/timestamp_policy_map.toml`
