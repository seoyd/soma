# Determinism guard

Sprint 23 adds deterministic fingerprints and ordering helpers for core reports.

## Included pieces

- `DeterminismInputFingerprint`
- `DeterminismOutputFingerprint`
- `DeterminismCheck`
- `stable_hash_string(...)`
- stable string ordering
- stable reason-code ordering
- deterministic float formatting

## Guard rules

- no wall-clock output unless explicitly passed
- env var names may appear, env var values must not
- file lists must be sorted deterministically
- reason-code rendering must be stable

This is meant to stop hidden nondeterminism from leaking into readiness or audit reports.
