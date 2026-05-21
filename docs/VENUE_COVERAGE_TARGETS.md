# Venue coverage targets

Sprint 25 separates official evidence coverage into three venue groups:

- `Crypto`
- `KoreanEquity`
- `USEquity`

## Why Upbit-only stays crypto-only

Upbit provides real official crypto candles, but it does not justify any claim about Korean or US equity readiness. A report with only Upbit-ready entries therefore remains `CryptoOnly`.

## Coverage targets

Each target declares:

- minimum ready datasets
- minimum outcome records
- minimum symbols
- minimum timeframes
- whether the target is required

Coverage passes only when the ready official entries for that venue meet every threshold and the venue is not auth-blocked.

## Weak evidence handling

One symbol or one thin slice of official data is still weak evidence. Sprint 25 keeps this conservative by marking such cases as partial or weak instead of over-claiming multi-venue readiness.

## Auth-aware interpretation

- missing KRX auth blocks `KoreanEquity`
- missing AlphaVantage auth blocks `USEquity`
- skipped collection entries with missing auth are reflected in the coverage summary
- mock fixtures are excluded from official claims

## CLI

```bash
cargo run --bin soma_experiment -- official-coverage --config examples/soma_venue_coverage_targets.toml
```
