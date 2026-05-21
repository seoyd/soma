# Official Candle Gap Map

Sprint 43 adds a deterministic gap map that turns comparable-evidence rows plus a candle pack into explicit candle shortfalls.

## Gap types
- `MissingOfficialCandleSeries` / `MissingNonCryptoOfficialCandleSeries`: official non-crypto coverage is still absent.
- `MissingProvenance` / `MissingPreflight`: local files exist but are not yet official-ready.
- `TimestampMismatch` / `TimeframeMismatch`: coverage exists but cannot be promoted conservatively.
- `MissingFutureWindow`: coverage exists but the required horizon is still too short.
- `ResearchOnlySource`, `FixtureOnlySource`, `ControlledOnlySource`, `CryptoOnlySource`: source boundaries remain explicit and never self-promote.

## Source boundaries
- non-crypto official rows are the only rows that can satisfy non-crypto official closure.
- crypto-only stays crypto-only.
- controlled evidence stays diagnostic-only.
- yfinance stays research-only.
- fixture and synthetic rows stay architecture-test-only.

## Routing
The gap map records whether each gap can be handled by:
- existing local canonical CSV reuse,
- bounded provider collection, or
- operator action for missing auth, approval, provenance, preflight, or local CSV delivery.

## Command
```bash
cargo run --bin soma_experiment -- candle-gap-map --config examples/soma_candle_gap_map_official_replication.toml
cargo run --bin soma_experiment -- candle-gap-map --config examples/soma_candle_gap_map_crypto_only.toml
```

The example gap-map configs assume a comparable bundle has already been written under `target/` by the Sprint 41 comparable-evidence flow.
