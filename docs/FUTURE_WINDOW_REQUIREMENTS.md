# Future Window Requirements

Sprint 46 adds a deterministic audit for official-ready rows that still cannot produce safe outcome references.

- An official-ready match can still lack outcome linkage when the local candle series does not contain enough bars after the decision timestamp.
- The report makes `required_start_timestamp_ms`, `required_end_timestamp_ms`, `required_future_bars`, and `missing_future_bars` explicit per row.
- Horizon is measured in future bars after the entry timestamp.
- Rows marked `no_lookahead_safe = false` are rejected instead of repaired.
- Controlled evidence stays diagnostic-only, yfinance stays research-only, fixture evidence stays architecture-test-only, and crypto-only evidence stays crypto-only.
- A sufficient future window improves evidence plumbing only; it does not prove profitability or live readiness.
