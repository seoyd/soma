# Official Candle Join Audit

Sprint 44 adds a deterministic join audit for explaining why comparable/scenario rows do or do not become official-ready candle matches.

## Inputs
- comparable evidence bundles
- official candle coverage pack
- official candle gap map
- official candle expansion report
- optional explicit symbol alias, timeframe alias, and timestamp policy maps

## Outputs
- `MatchKeyNormalizationAggregate`
- `RowCandleCandidateReport`
- `GapExpansionConsistencyReport`
- `OfficialCandleLineageReport`
- local JSON and text renderings under the configured output directory

## Safety rules
- local paths only
- research-only / paper-only usage
- explicit maps only; no fuzzy matching
- source class cannot be promoted through normalization
- no-lookahead rejection remains blocking

## Command
```bash
cargo run --bin soma_experiment -- candle-join-audit --config examples/soma_candle_join_audit_official_replication.toml
```
