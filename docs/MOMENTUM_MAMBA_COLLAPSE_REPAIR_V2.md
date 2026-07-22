# Momentum Mamba Collapse Repair V2

## Boundary

This is an offline, additive research iteration over the verified 312-row
Momentum snapshot. The migrated intent and view, V1 session, projection,
participants, qualification receipts, family, usage ledger, blocked
registration, prospective artifacts, and active three-agent state remain
immutable. Historical-test and future-evaluation evidence are not opened.

The failed V1 participant is bound by digest `22632d7a5f0e1ab2` and its source
family by digest `72cd657ea8a1f039`. The audit replays only the already-consumed
V1 training and validation ranges. It does not inspect the range previously
classified as `ReservedRetrospectiveUnused`.

## Collapse audit

The machine-verifiable audit classified the reproduced failure as
`ProbabilitySingleSided`. Representation and head-optimization diagnostics
were recorded independently and were not classified as the root cause. No
numerical failure was observed. Public proof identities are:

```text
collapse_audit_digest = 5f047849bef0a026
representation_diagnostic_digest = 2ab09dee0d95ceb7
optimization_diagnostic_digest = 11a514c7bdb9f5ac
probability_diagnostic_digest = a6230ff281c5def5
class_balance_diagnostic_digest = 8b04b5aec502d65f
repair_capability = RepairableWithBoundedHeadRegularization
```

The diagnostic side of the implementation separately derives finite and
variance classifications per representation dimension, constant-dimension and
effective-rank status, unique representation count, head parameter-delta and
gradient-norm classes, update count, schedule and loss-trajectory identities,
early-stop reason, probability variance and entropy classes, unique bins,
single-side and saturation classes, extrema identities, and training/prior
validation class-balance counts. Raw diagnostic values are private.

## Fresh split and preregistration

The prior unused range was `[96, 312)`. The deterministic repair split is:

```text
repair training = [0, 160)
repair purge = [160, 176)
fresh repair validation = [176, 200)
remaining reserved = [200, 312)
split_digest = e9029f489e01a87b
```

The 16-row purge covers the feature history, sequence length, and label
horizon. Training labels stop before the purge, validation labels remain within
the admitted historical boundary, and prior-validation, prospective, and
future-evaluation overlap counts are zero. The fresh validation range was
previously unused and is shared exactly by every V2 participant.

Registration `68eb18d97c06f4f3` was persisted and reopened before fresh
validation inference. It froze three configurations:

| Variant | Pooling | Configuration digest |
| --- | --- | --- |
| `v1-control` | `LastOutput` | `79f4d70dfbed9760` |
| `l2-regularized` | `LastOutput` | `fe7dca3d5adc4081` |
| `low-rate-l2` | `LastOutput` | `c3ed36386e8fc092` |

Every variant creates a fresh deterministic head, fits its feature normalizer
on repair training only, and keeps the encoder frozen. V1 head reuse and warm
start are false. No result-dependent second batch can be added without changing
the preregistration identity.

## Fresh validation result

All participants used the same fresh validation timestamp identity:

| Participant | Digest | Qualification |
| --- | --- | --- |
| `FrozenMambaHeadV2/v1-control` | `5a5ad7e6b7ebe46c` | `RejectedProbabilityCollapse` |
| `FrozenMambaHeadV2/l2-regularized` | `48449bbbdbf0c27e` | `RejectedProbabilityCollapse` |
| `FrozenMambaHeadV2/low-rate-l2` | `cd15ef353c46d2f3` | `RejectedProbabilityCollapse` |
| `LinearMomentumBaselineV2` | `3bdd2cc155834480` | `Qualified` |
| `ConstantProbabilityBaselineV2` | `fac4d794b05257cd` | `BenchmarkQualified` |

The constant benchmark is role-aware and is not rejected merely for intentional
zero probability variance. Learned candidates retain the non-collapse gate.
Validation parameter updates and historical-test/future-evaluation reads are
zero.

Family `fb7d3825c2ae8911` retains all five participants and all qualification
receipts. It has zero qualified learned participants and two qualified
comparators. No winner was selected, and active, promotion, and reward
eligibility remain false.

Because no learned participant qualified, roster status and evaluation
registration status are both `NoQualifiedLearnedParticipant`. Linear versus
Constant was not registered by itself, no minimum future timestamp was
assigned, and no future evidence was acquired.

## Persistence and authority

The first execution wrote 15 manual-Protobuf sidecars: audit, split,
preregistration, five participants, five qualification receipts, family, and
journal. The roster and future registration codecs are implemented and tested
but their artifacts are absent because the learned-participant gate did not
pass. Every write uses create-new temporary storage, flush, `sync_all`, temporary
reopen and validation, atomic rename, and final reopen and validation. The
second execution wrote nothing and duplicate-rejected all 15 sidecars.

All network, transport, credential, prospective-row, label-opening,
future-evaluation, historical-test, active-model, Chair, vote, reward, penalty,
voice, cooldown, promotion, quarantine, and execution counters are zero. The
active committee count remains three.

The offline command is:

```text
--momentum-mamba-repair-v2 --status|--dry-run|--execute-local
--output-format text|json
```

Every mode rejects network permission and authority flags. Status and dry-run
write nothing. Public output contains classifications, identities, statuses,
counts, optional future-registration boundaries, and zero safety counters only.
It does not expose rows, representations, logits, probabilities, labels,
metrics, parameters, gradients, or local paths.

This result proves enforcement of the bounded repair and registration gates. It
does not prove model improvement, participant superiority, promotion readiness,
reward effectiveness, Chair learning, or trading readiness.
