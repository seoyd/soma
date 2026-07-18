# Owner Snapshot V2 Replay

Sprint 59 used the existing repository-local ignored campaign configuration and
its ignored immutable Protobuf historical snapshots. No snapshot was created,
downloaded, altered, copied, or committed.

The current CLI decoded the available local snapshots, deterministically selected
the configured campaign evidence, and failed closed unless the configuration and
snapshot invariants held. The accepted parent evidence verified its Protobuf
identity and content digest, strict chronology, dataset and quality row counts,
single-symbol metadata, timestamp metadata, read-only and sanitized state, and
historical immutable provenance. The V2 registration reused the two existing V1
scope identities without changing ranges, cutoffs, rows, participant
configuration, candidates, features, labels, thresholds, seeds, or policies.

For each registered scope, V2 materialized an in-memory immutable child snapshot
with a distinct semantic identity, coupled metadata, parent-child lineage, and
exact-child authorization. Parent evidence remained unchanged. The replay was
offline and credential-free: provider calls, transport construction, network
consent reads, and credential reads were all zero.
