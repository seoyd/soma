# Outcome Linkage V3

Outcome linkage v3 converts a safe future window into a deterministic triple-barrier outcome reference.

- Entry is the close at the decision timestamp.
- Take-profit and stop-loss barriers are applied over the configured future horizon.
- Cost and slippage are subtracted from gross return before the stored net return is written.
- Tie-breaks are deterministic through the configured `TripleBarrierTieBreakPolicy`.
- Rows without enough future bars, with timestamp drift, or with no-lookahead violations are skipped or rejected instead of backfilled unsafely.
- Generated outcomes improve evidence plumbing only; they do not prove model usefulness or profitability.
