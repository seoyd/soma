# Cross-agent scope alignment

## V1 provenance gate

V1 mapping may use only opinions with a uniquely verified immutable source.
When a legacy opinion can be reconstructed from zero or multiple Risk results,
it remains visible but is excluded from one-to-one mapping and aggregation.

Source-bound V1 opinions also require exact canonical raw-scope alignment
before pairing. When historical Momentum regimes and Risk half-ranges differ,
the registry records `SourceBoundButScopesNotComparable`; it does not truncate
evidence, synthesize an intersection, or create a deliberation.

Momentum and Cycle/Risk scopes are reconstructed independently. Raw-row
alignment, effective-anchor alignment, and objective-horizon alignment are
separate records. A pair may be compared only through matching canonical raw
scope identity, not display names or chronology indexes.

The registry is deterministic and preserves unmatched or ambiguous opinions.
Aggregate deliberation is composed only for a complete one-to-one mapping; it
counts existing relationship classifications and abstentions only. It never
averages probabilities, metrics, or confidence and never produces an action.
