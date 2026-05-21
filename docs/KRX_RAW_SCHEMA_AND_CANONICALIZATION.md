# KRX Raw Schema and Canonicalization

Sprint 50 archives bounded KRX raw responses and validates the response schema before canonicalization.

## Raw response archive

- fixture replay stores a copied local JSON response under the closure output root
- request metadata stays redacted
- auth-bearing values are never written to disk

## Schema drift

The schema drift report checks that each raw response contains:

- `symbol`
- `timeframe`
- `rows`
- per-row `date`, `open`, `high`, `low`, `close`, `volume`, `trade_value`, `bid`, `ask`, `spread_bps`

Missing required fields, bad dates, bad prices, bad volume, empty payloads, or unsupported shapes block canonicalization.
Unexpected fields are reason-coded without panicking.

## Canonical CSV

Canonical KRX CSVs use:

- `timestamp_ms`
- `open`
- `high`
- `low`
- `close`
- `volume`
- `trade_value`
- `bid`
- `ask`
- `spread_bps`

No silent field guessing is allowed. If required raw fields are missing, Sprint 50 stops rather than inventing replacements.
