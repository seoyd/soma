# Canonical learned-agent scope identity

Sprint 54 adds an agent-neutral identity for immutable historical observation
scopes. A scope digest includes provider and series identity, the semantic
snapshot digest, segmentation policy, canonical row-set digest, canonical row
order digest, and information cutoff. Row identity includes timestamp, symbol,
and OHLCV fields; duplicate canonical rows are rejected.

The identity is external to learned opinions. It does not revise model results,
opinion digests, seals, arguments, or transcripts. Scope names, array order,
row count, and timestamp ranges are not mapping keys.
