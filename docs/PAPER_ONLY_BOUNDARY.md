# Paper-Only Boundary

The Toss adapter is an input adapter. It can produce existing market and risk
snapshots, but it cannot produce, submit, modify, or cancel an order.

The boundary is structural:

- `TossCapabilities.order_execution` and `live_execution` are always false.
- `TossClient` has no order method.
- Only registered, validated, callable read-only contracts can reach
  `TossTransport`.
- Account and candle contract shapes remain disabled until private schemas are
  reviewed locally.
- Token auth is a disabled placeholder, not a trading endpoint.
- Order, cancel, unknown, and mutated non-read-only contracts are rejected
  before transport invocation.
- The adapter returns `NoTrade` as its default action.
- API failure sets API health and data quality to zero.
- `RiskGovernor` remains the final absolute veto.
- Owner input is advisory and cannot force Risk Governor approval.
- An approved decision can reach only the existing `PaperBroker`.
- `PaperBroker::supports_live_execution()` remains false.

The adapter cannot bypass the signal path, three delegates, Chair, or Risk
Governor because it returns data inputs only. The existing
`simulate_paper_cycle` owns the full decision sequence and the only execution
handoff.

Real execution would require a separate sprint and explicit approval. That work
would need an independently reviewed broker module, a default-disabled
compile-time feature, credential-scope review, idempotency and reconciliation,
additional audit controls, and new failure-mode analysis. None of those
capabilities are present in this sprint.
