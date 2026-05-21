# Soma Control Tower UI

Soma Control Tower v0 is a **read-only, local-only** monitoring UI.

It renders static HTML/JSON/TXT snapshots for:
- provider state,
- evidence status,
- committee state,
- chair decision,
- risk governor,
- candidate queue,
- paper positions,
- human confirmation queue,
- bottlenecks,
- audit timeline.

Rules:
- no live trading,
- no broker/order/account controls,
- no external CDN/assets,
- no remote outputs,
- no runtime LLM,
- no persona expansion.

`dashboard-open` and `dashboard-serve` remain deferred in Sprint 52.
