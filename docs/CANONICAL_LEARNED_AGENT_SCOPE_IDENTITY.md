# Canonical learned-agent scope identity

## V1 external provenance

V1 adds a byte-level canonical encoding for new audit artifacts, exact
IEEE-754 row identities, deterministic Risk frozen-pack range resolution, and
audit-only partitioned anchor identities. V0 identity remains preserved for
historical readability and compatibility; it is not upgraded in place.

Sprint 54 adds an agent-neutral identity for immutable historical observation
scopes. A scope digest includes provider and series identity, the semantic
snapshot digest, segmentation policy, canonical row-set digest, canonical row
order digest, and information cutoff. Row identity includes timestamp, symbol,
and OHLCV fields; duplicate canonical rows are rejected.

The identity is external to learned opinions. It does not revise model results,
opinion digests, seals, arguments, or transcripts. Scope names, array order,
row count, and timestamp ranges are not mapping keys.
