# Derived Scope Snapshot Integrity

A joint scope creates a distinct immutable child snapshot. Its content digest and
snapshot ID are calculated from exactly the registered chronological row subset;
the child never reuses the parent identity after its content changes.

The derivation proof binds parent identity, registered scope identity, child
identity, row counts, quality-summary row count, symbol, timestamps, content digest,
read-only state, sanitization, credential-free provenance, immutable reason code,
and parent-child lineage. Any failed invariant blocks the V2 replay.

Evidence authorization is exact-child only. The derived policy authorizes the one
verified child snapshot ID from the proof, forbids wildcard authorization, and does
not mutate the global historical-evidence policy.
