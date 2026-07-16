# BTC multi-regime evidence

The BTC Momentum experiment remains an offline, CPU-backed ShadowOnly research
prototype. This evidence path answers a temporal replication question within one
BTC daily series; it is not independent-market, cross-provider, or equity
replication.

Accepted history is immutable. A bounded approved daily-source backfill may run
only after explicit local network consent, and it creates a new Protobuf V1
snapshot after deterministic merge, reload, semantic-digest, and identity
verification. The prior snapshot is never mutated. Conflicting duplicate bars
are rejected; identical bars are deduplicated; missing days are not invented.
Before any such call, the runner derives the required chronology coverage and
minimum whole-page count, validates a bounded request budget, and emits a
sanitized dry-run digest. It uses one sequential request at a time and stops on
429 or permission failure without retry.

Every accessed historical range is permanently classified in the deterministic
usage ledger: training, validation, test, diagnostics, counterfactual, or
development eligibility. The ledger contains timestamps and immutable IDs only;
it never stores raw rows or local paths. Consumed evidence is useful for
regression and replication, but cannot become a pristine final holdout.

Regimes use a validated equal-length chronological policy, configured row count,
and optional inter-regime row gap. Boundaries are computed from chronology
before campaign execution and cannot use labels, model outputs, or diagnostics.
Each valid regime becomes one independently frozen single-series evidence pack;
packs may not share rows.

Each pack runs the unchanged C0–C3 Momentum campaign with unchanged feature
schema, encoder, optimizer, gates, thresholds, pooling, and CPU backend policy.
Per-regime reports retain no-signal outcomes, support decisions, temporal root
causes, warm/cold status, abstentions, and ShadowOnly boundaries. Research-only
counterfactual results are recorded as consumed evidence and never promote a
model.

Cross-regime aggregation can call frozen-representation risk recurrent only
after the configured number of independent chronological packs shows that stage
without an earlier feature or sequence explanation. Sparse, no-signal, mixed,
or insufficient regimes remain visible and cannot be called cross-market
evidence.
