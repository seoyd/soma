# Official Candle Coverage Pack

Sprint 42 adds a deterministic, local-only candle pack for coverage checks.

## Official requirements
- local file paths only
- bounded `max_rows`, `max_symbols`, `max_timeframes`, and `max_bytes`
- official non-crypto readiness requires official source classification, local provenance, ready preflight, and no duplicate/gap regression
- crypto official series stay crypto-only

## Source boundaries
- `OfficialNonCrypto` can support official-readiness matching
- `OfficialCryptoOnly` can support crypto-only benchmarking, not non-crypto official claims
- `ControlledDiagnostic`, `YFinanceResearch`, `FixtureArchitectureTest`, and `SyntheticTest` stay diagnostic-only
- filename hints do not upgrade source class

## Sidecars
- provenance proves local origin and source boundary
- preflight proves the file was locally checked and is ready for real-evidence use
- manifest is optional but preserved when present

## Command
```bash
cargo run --bin soma_experiment -- candle-pack --config examples/soma_candle_pack_official_controlled.toml
```

Research-only. No live trading, broker, account, or runtime-LLM path is added.
