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

## V1 family evidence ledger

V1 does not reinterpret the V0 ledger. It constructs a new ledger from the
validation-only execution with explicit entries for view binding, trainer
projection, feature derivation, label derivation, normalizer fit, parameter
training, validation inference, validation metrics, family inclusion,
referenced-but-unconsumed evidence, and reserved retrospective rows. Training
and validation ranges are distinct, the purge range is unconsumed, and all
remaining historical rows are `ReservedRetrospectiveUnused`.

The V1 ledger rejects any nonzero historical-test row read, label read,
inference, metric, checkpoint-selection, or identity-influence value. Family
identity includes only frozen participant and lineage semantics. Validation
metric digests live exclusively in qualification receipts and therefore cannot
alter participant parameters or family identity.

## Explicit future-evidence exclusion

The V1 exclusion contract binds the existing protected opening registration,
the protected Momentum and Cycle/Risk capsule identities, and every timestamp
already reserved by that lane. Reservation metadata is parsed without opening
outcome rows, labels, probabilities, or metrics. For the current protected
metadata the derived exclusion set contains four timestamps. A capsule
admission check rejects any excluded timestamp even if a scalar candidate
cutoff would otherwise allow it.

The minimum accepted timestamp is calculated from the maximum legal next
timestamp after the candidate source end, the final reserved timestamp, all
protected boundaries, and the provider-finality boundary. The production code
does not hardcode a calendar result. It advances by the protected cadence and
accepts only timestamps at or after the derived minimum.

## V1 registration

A registration is created only when the V1 session, complete view, projection,
family, qualification receipts, usage ledger, and exclusion contract all
verify; at least two frozen participants exist; every included participant is
qualified; historical-test access is zero; and no winner was selected. One
agent's blocker does not affect another.

Every registered participant and qualification-receipt digest is sorted and
frozen before future evidence. Maximum requests and concurrency are one,
retries are zero, labels and probabilities stay hidden until a one-time
opening, and winner selection, promotion, and reward application remain
forbidden. This Sprint performs no request and opens no evidence.

The exclusion, registration, and registration journal use manually defined
Protobuf artifacts under `state/learning_data/evaluation_v1/{agent}` with
temporary write, flush, `sync_all`, temporary reopen, atomic rename, and final
reopen verification. Repeated identical writes are duplicate-rejected.

The offline CLI is:

```text
--register-agent-candidate-evaluation-v1 --status|--dry-run|--execute-local
--output-format text|json
```

With the current local evidence, Momentum and Cycle/Risk have no complete V1
view and are reported as explicit candidate-unavailable blockers;
Value/Quality remains candidate-unavailable by capability. No future
performance, winner, promotion, reward, or trading-readiness conclusion follows.

## Sprint 75 canonical-view rerun

The canonical-view audit now precedes V1 candidate and evaluation registration.
It derives each gap from persisted intent identity, current policy, immutable
evidence, trainer capability, and verified provider semantics. A successful
view for one agent may proceed independently; another agent's missing evidence
does not become a global blocker.

The current offline rerun produced no V1 family. Momentum remained
`InsufficientEvidence`, Cycle/Risk remained `InsufficientEvidence`, and
Value/Quality remained `TrainerUnavailable`. Their evaluation registration
statuses were `CandidateUnavailable`. Historical-test access stayed zero, no
winner was selected, and no family became eligible for committee membership,
promotion, reward, or active mutation.

No future evaluation evidence was opened. The four protected timestamps,
protected registration identities, candidate-source boundary, and provider
finality boundary remain mandatory exclusions for every future V1 registration.
Both boundary audits passed.
