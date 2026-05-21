# KRX Symbol Whitelist

Sprint 49 uses a compact whitelist for Korean equity symbols.

## Rules

- no wildcard
- no all-symbol scan
- deterministic ordering
- normalized symbols are six-digit uppercase strings
- provider symbols are preserved separately from normalized symbols
- disabled or invalid entries are skipped and reason-coded

## Example

`examples/sprint49_data/krx_whitelist_compact.toml` keeps the scope bounded to a small local fixture set.

## Why it matters

The whitelist keeps KRX collection/import bounded by symbol count, rows, requests, days, and bytes so the activation flow stays local-first and research-only.
