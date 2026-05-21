# KRX Market-Data-Only Boundary

For Sprint 91, KRX evidence remains market-data-only:

- no broker path,
- no order path,
- no account path,
- no balance or holdings path,
- no correction/cancel path,
- no broker execution path.

Operator-facing auth and endpoint-template actions may mention env-var names, but never secret values.
