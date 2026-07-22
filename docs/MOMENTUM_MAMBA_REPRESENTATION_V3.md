# Momentum Frozen Mamba Representation V3

Momentum representation V3 is an offline, additive experiment over the frozen
V1 and V2 research history. It does not rewrite earlier artifacts, mutate the
active committee, open historical-test or future-evaluation evidence, or apply
rewards and penalties.

## Frozen history and probes

Execution first reopens and validates the exact V1 collapsed participant and
the exact V2 head-only repair result. V2 must retain three
`RejectedProbabilityCollapse` learned participants, two role-qualified
comparators, no winner, no roster, no future registration, and no active or
reward authority. This establishes `V2HeadOnlyRepairExhausted` and forbids a
result-dependent learning-rate, regularization, seed, or epoch sweep.

The representation-path audit uses only V2-consumed training and validation
ranges. It evaluates four deterministic probes: raw-feature Linear, frozen
Mamba Last output, frozen Mamba Mean output, and frozen Mamba Last+Mean
concatenation. Probe artifacts expose statuses and diagnostic digests only;
private rows, representations, probabilities, labels, metrics, and parameters
remain private. The audit records that fresh V3 validation was not accessed.

The verified probe result was:

| Probe | Status | Representation diagnostic |
|---|---|---|
| Raw feature | `NonCollapsedPrediction` | `3803b764eb6cbf30` |
| Last output | `SingleSidedPrediction` | `69646c99b529112f` |
| Mean output | `SingleSidedPrediction` | `8bea1baa802dbd81` |
| Last+Mean | `LowEffectiveRank` | `bc35b8d2d745fc2c` |

The representation audit digest is `190d01aaf87681a9`.

## Fresh split and preregistration

The V3 split is derived from the remaining V2 reserve and the existing feature
history, sequence length, label horizon, and minimum-validation policy. The
verified ranges are:

```text
training        [0, 224)
purge           [224, 240)
fresh validation[240, 264)
final reserve   [264, 312)
```

The purge supplies feature and sequence context, while V3 qualification selects
exactly the 24 labels inside the fresh-validation range. No label at or after
index 264 is built or read. The split digest is `44828a88d5ae2c11`; the final
reserve identity digest is `9b1b9f400b429b83`.

Before fresh-validation inference, execute-local atomically persists and
reopens registration `6250b81ff35d8dd8`. It freezes exactly four learned
routes: Last, Mean, Last+Mean, and Last plus raw-feature residual. Every route
uses a frozen encoder, a fresh deterministic logistic head, training-only
normalizers, the same validation timestamps, and zero validation updates. V1
and V2 heads cannot be reused. Linear and Constant are freshly fit on the same
training and validation ranges.

## Qualification and contribution

Mamba-only routes require finite, sufficient, non-collapsed validation results
and valid representation diagnostics. The residual route additionally uses a
deterministic block-zero ablation. Its registration-bound contribution policy
classifies Mamba and raw block effects without publishing their values.
`QualifiedRawFallbackNotMamba` is retained as evidence but cannot satisfy the
genuine-Mamba roster gate.

The verified qualifications were:

| Participant | Qualification | Contribution |
|---|---|---|
| Last output | `RejectedRepresentationInvariant` | `NotApplicable` |
| Mean output | `RejectedRepresentationInvariant` | `NotApplicable` |
| Last+Mean | `RejectedRepresentationInvariant` | `NotApplicable` |
| Last+raw residual | `RejectedRepresentationInvariant` | `MaterialContribution` |
| Linear comparator | `ComparatorQualified` | not applicable |
| Constant benchmark | `BenchmarkQualified` | not applicable |

The residual contribution classification does not override its failed base
representation qualification. Consequently the family has zero qualified
genuine-Mamba routes and zero qualified raw fallbacks. All six participants,
six receipts, and four contribution audits remain in family
`afc3aa14cc1622da`.

The route decision is `AllRepresentationRoutesCollapsed`, digest
`106e0e00dd7506f9`. The frozen-Mamba representation path is terminal for this
contract. No future roster or evaluation registration exists, and no minimum
accepted future timestamp is assigned.

## Persistence, CLI, and safety

Twelve artifact categories use hand-written `prost::Message` codecs and the
existing verified atomic writer: probes, audit, split, registration,
participants, qualification receipts, contribution audits, family, decision,
optional roster, optional evaluation registration, and journal. The applicable
terminal result contains 26 Protobuf sidecars. Repeated execution writes zero
new files and duplicate-rejects all 26 semantic identities.

The command is:

```text
--momentum-mamba-representation-v3 --status|--dry-run|--execute-local
--output-format text|json
```

Exactly one mode is required. All modes reject network permission and authority
flags before local I/O. Status and dry-run write nothing. Public text and JSON
contain only contract versions, statuses, digests, counts, blockers, replay
eligibility, and safety counters.

Network, transport, credential, prospective-row, prospective-label,
historical-test, future-evaluation, active-model, Chair, vote, reward, penalty,
voice, cooldown, promotion, quarantine, and execution counters are zero. The
active committee count remains three. Cycle/Risk remains
`ProviderContractUnverified`; Value/Quality remains `TrainerUnavailable`.
Persisted prospective attribution and reward eligibility replay unchanged, with
zero reward and penalty applications.

This result proves the bounded offline representation-path protocol and its
terminal outcome for the frozen evidence. It does not prove model improvement,
participant superiority, future performance, promotion readiness, reward
effectiveness, Chair learning, or trading readiness.
