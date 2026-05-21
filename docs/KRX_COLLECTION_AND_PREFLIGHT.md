# KRX Collection and Preflight

Sprint 49 treats KRX evidence as local-first.

## Preferred order

1. reuse local canonical CSV
2. reuse existing bounded collected CSV
3. plan KRX collection only when explicitly enabled and auth is ready

## Required artifacts for official readiness

- canonical CSV
- provenance JSON
- preflight JSON
- manifest when generated from preflight

## Bounds

- `max_symbols <= 5`
- `max_rows_per_symbol <= 300`
- `max_requests <= 10`
- `max_days <= 365`
- storage budget must remain bounded

## Failure cases

- missing provenance blocks official readiness
- missing preflight blocks official readiness
- bad timestamps, duplicate timestamps, or OHLC invariant failures block readiness
- budget overflow triggers warnings or blocked collection status
- remote paths are rejected

## Notes

Tests never run the real KRX network path. Local fixtures are used for smoke coverage and determinism checks.
