# Control Tower Model Ops

Sprint 65 adds a static **Control Tower model ops panel** for external model research review.

The panel summarizes:

- research ops status
- review queue counts
- watchlist entries
- comparability status
- artifact completeness summary
- evidence risk profiles
- leaderboard changes
- Mamba deferred state
- copyable local CLI commands

The panel intentionally omits:

- train buttons
- live buttons
- broker/order/account controls
- runtime Mamba controls

It remains a read-only local artifact, not an execution surface.
