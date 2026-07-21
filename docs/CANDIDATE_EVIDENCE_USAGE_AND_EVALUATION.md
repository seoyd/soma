# Candidate Evidence Usage and Evaluation V0

## Purpose

This protocol audits how each private Shadow candidate used retrospective
evidence and records a future, non-overlapping evaluation contract. It is
offline-only. It neither executes the future evaluation nor changes active
state.

## Lineage admission

The audit reads one explicitly bounded candidate directory per known agent. A
candidate is admitted only when there is exactly one candidate envelope, one
matching session, one matching private dataset manifest, and a valid trainer
projection. Semantic digests, agent ownership, session/view/source bindings,
candidate safety flags, and Protobuf envelopes are verified before use.

Hardened sessions load their persisted projection. A preliminary session may be
audited only when its effective trainer input can be reconstructed without
ambiguity from one source artifact. Preliminary sessions do not satisfy the
complete source-policy and projection binding required for future registration;
they remain immutable retrospective records.

## Evidence-usage ledger

Each consumed artifact receives digest-bound entries for intent and view
binding, trainer projection, feature and label derivation, normalizer fitting,
parameter training, validation inference and metrics, checkpoint selection,
historical-test inference and metrics, and candidate identity. Authorized
artifacts present in the complete view but not consumed by the trainer are
recorded as `Unused`, rather than silently flattened into the trainer input.

Ranges come from the persisted trainer manifest and are interpreted together
with the actual Momentum or Cycle/Risk result construction. This matters because
both candidate constructors include test-bearing result metrics in
`private_metrics_digest`, and that digest contributes to the candidate identity.
No status is inferred merely from a range name.

## Historical-test and identity audit

The current local audit produced:

| Agent | Historical-test status | Candidate-identity result |
| --- | --- | --- |
| `momentum_trend_fast` | `InfluencedCandidateIdentity` | Test-bearing private metrics contribute to the candidate digest. |
| `cycle_risk_skeptic` | `InfluencedCandidateIdentity` | Test-bearing regime results contribute to the candidate digest. |
| `value_quality_filter` | `NoCandidate` | Trainer and candidate remain unavailable. |

These results do not establish candidate improvement and do not make either
historical test reusable as a pristine test. The audits are additive artifacts;
the original candidate envelopes are unchanged.

## Future evaluation registration

The cutoff is derived as the maximum of the candidate session cutoff, dataset
manifest cutoff, and any supplied protected prospective-lane boundary. Only
timestamps strictly greater than the resulting exclusive cutoff can be
admitted. Training, purge, validation, historical-test, candidate-identity, and
reserved prospective evidence therefore cannot be reused.

Comparator identities are frozen from existing lineage before future evidence
is available. A real parent is included only when the candidate already names
one. No missing parent, linear baseline, constant baseline, or Cycle/Risk
comparator is invented.

The current preliminary candidates are `PolicyInvalid` for registration because
their persisted sessions predate complete input-policy and trainer-projection
binding. Their derived exclusive cutoff is `1784073600000`. Momentum has one
existing parent comparator; Cycle/Risk has none. Value/Quality has no
registration because it has no candidate. These are registration outcomes, not
performance results.

Every registration fixes these safety boundaries:

- maximum requests and concurrency are one;
- maximum retries are zero;
- labels and probabilities remain hidden until a one-time opening;
- active promotion and reward application are forbidden;
- no future row, label, prediction, or metric is created in this Sprint.

## Persistence and repeatability

Trainer projections, evidence-usage ledgers, identity audits, registrations,
and registration journals are manually defined Protobuf artifacts under
`state/learning_data/evaluation/{agent}`. Semantic identity is independent of
encoded bytes. Writes use a temporary file, flush, `sync_all`, temporary reopen
and semantic verification, atomic rename, and final reopen and verification.

An identical second registration execution reported ten duplicate artifacts
and zero storage failures. Existing learning artifacts retained their original
SHA-256 values.

## CLI

```text
--agent-candidate-evidence-audit --status|--dry-run|--execute-local
--register-agent-candidate-evaluation --status|--dry-run|--execute-local
--output-format text|json
```

Exactly one command and one mode are accepted. Network permission is rejected.
The public result contains only identities, statuses, cutoff, comparator count,
storage counts, and zero safety/authority counters. It excludes raw rows,
labels, metrics, model parameters, predictions, probabilities, and local paths.
