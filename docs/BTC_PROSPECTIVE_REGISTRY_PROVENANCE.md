# BTC prospective registry provenance

The prospective challenge registry has two distinct legal pre-accumulation
states. `Sealed` is the local capsule state before the public pre-registration
boundary. `PreRegistrationCommitted` is the same immutable challenge after
that boundary has been confirmed. Their registry digests are intentionally
different because state is part of the registry semantic identity.

The registry does not replace an earlier digest. It stores an append-only
transition record binding the challenge and capsule identifiers, previous and
next state, previous and next registry digests, before/after vault and journal
digests, reason, and evidence/label access flags. A Phase A transition has no
evidence access and no label access. Its deterministic transition digest lets a
later verifier reconstruct why the two digest values differ.

Registry provenance closes only when the capsule validates, the uniquely
resolved candidate and both frozen comparators equal the sealed artifacts, the
cutoff and all frozen policies equal the re-derived capsule, the vault and
journal are empty, and the transition chain reaches the local current registry
digest. A missing record, changed capsule, invalid chain, or unexpected digest
blocks all future acquisition.

The local state remains ignored and is written atomically then reopened and
validated. A legacy committed local state with no transition record may receive
one reconstruction record only when its immutable capsule, empty vault, empty
journal, and deterministic sealed/current registry digests verify. It does not
regenerate a capsule or rewrite either registry state.

After provenance closure, later legal transitions bind any vault or journal
digest change to a named reason. Future data cannot be requested before this
closure. An unexplained change preserves local artifacts, blocks acquisition,
and requires a separate challenge rather than repair by overwrite.
