# Momentum V4 One-Time Supplemental Qualification

Momentum V4.1 is an offline, additive qualification pass over the already
frozen V4 family. It reuses the exact two learned heads, constant benchmark,
feature policy, label policy, training-only normalizers, optimizer settings,
initialization seeds, and qualification policies. It neither retrains nor
changes a participant.

## Frozen reconstruction and preregistration

The implementation reopens the V4 closure, split, registration,
validation-yield audit, family, corrected decision, and original receipts. It
then reconstructs each participant from the exact V4 training prefix and
registered configuration. Parameter, normalizer, model-artifact,
training-identity, participant, family, contribution-audit, and decision
identities must match the persisted V4 values. A mismatch stops the pass before
the reserve is opened.

Supplemental registration `0e9762c34cae048b` binds the source snapshot,
canonical intent and view, V4 identities, all three participant identities,
parameter and normalizer digests, the original validation range, the
supplemental range, and the unchanged minimum of 24 valid samples. It was
persisted and reopened before reserve access.

The ranges are derived from the persisted split:

```text
original validation     [264, 288)
supplemental validation [288, 312)
```

No supplemental row is used for training, normalization, configuration
selection, or parameter updates.

## One-time reserve opening

A `Ready` receipt is persisted after preregistration and frozen reconstruction
but before supplemental example or label construction. The successful
`Opened` receipt records one opening attempt and the exact 24-index reserve.
The opening receipts are additive and do not alter the V4 split or its
historical artifact.

An identical execution reopens the completed result as `AlreadyOpened`. It
performs zero new reserve row reads, zero new reserve label reads, zero new
model work, and zero writes. The verified replay duplicate-rejects all 13 V4.1
sidecars.

## Accumulated evidence and qualification

The original block contains 23 valid labelled examples and one neutral
exclusion. The supplemental block also contains 23 valid labelled examples and
one neutral exclusion. The exact, duplicate-free union therefore contains 46
valid examples and reaches the unchanged minimum of 24.

All predictions and private qualification metrics are recomputed directly over
that union with the frozen heads. Prior statuses and metric summaries are not
averaged. The resulting status names are:

| Frozen participant | Accumulated status |
|---|---|
| `RawFeatureLogisticV4` | `QualifiedLearned` |
| `RawFeatureInteractionLogisticV4` | `QualifiedLearned` |
| `TrainingPrevalenceConstantV4` | `BenchmarkQualified` |

The additive interaction audit is
`1701e17d2dc56f8d` and reports
`MaterialInteractionContribution`. This is a policy classification over the
accumulated evidence, not a ranking, winner, or improvement claim.

Accumulated family `4900d33cd7f0eb60` retains all three participants and their
additive receipts. It records two qualified learned participants, one qualified
benchmark, no winner, no parameter change, and no active-committee, promotion,
or reward eligibility.

The accumulated path decision is `RawFeatureLearnedPathViable`, digest
`d08e8a1d2ecfef4f`. The decision means only that the registered path meets its
bounded qualification policy on the accumulated evidence.

## Roster and future evaluation contract

Roster `4883ef775c6e5589` includes both qualified learned participants and the
qualified constant benchmark without private-metric ranking. The interaction
entry is admitted because its accumulated status is `QualifiedLearned` and its
contribution status is `MaterialInteractionContribution`.

Future evaluation registration `372b95d7dfee4bef` binds the original V4 family,
the accumulated family, registration and opening receipts, accumulated yield,
all accumulated receipts, interaction audit, full source boundary, consumed
V1–V4 validation identities, protected registrations and timestamps, and the
provider-finality boundary. Its minimum accepted timestamp is
`1784764800000`. Labels and probabilities remain hidden until a separate
one-time opening; request and concurrency maxima are one and retries are zero.
No future evidence was acquired in this pass.

Because the accumulated minimum was reached, no additional-evidence
requirement was created.

## Persistence, CLI, and safety

The command is:

```text
--momentum-v4-supplemental-qualification
--status|--dry-run|--execute-local
--output-format text|json
```

Status and dry-run write nothing and do not access the reserve. Every mode
rejects network and authority flags. Public output is limited to status names,
digests, counts, the future boundary, existing reward-eligibility replay, and
safety counters; it does not expose rows, returns, labels, probabilities,
metrics, parameters, gradients, features, or local paths.

Manual `prost::Message` contracts persist the supplemental registration, two
opening receipts, supplemental yield, three accumulated receipts, interaction
audit, accumulated family, decision, roster, future registration, and journal.
The first completed execution wrote 13 additive sidecars; repeated execution
wrote zero.

Network, transport, credential, new prospective, historical-test, and
future-evaluation reads are zero. The one authorized reserve opening read 24
rows and 24 labels. Participant parameter changes, active-model changes, Chair
decisions, votes, rewards, penalties, voice changes, cooldowns, promotions,
quarantines, and executions are zero. The active committee count remains three,
and all protected pre-existing artifacts and canonical agent states remain
unchanged.

Cycle/Risk remains `ProviderContractUnverified`, Value/Quality remains
`TrainerUnavailable`, and read-only prospective replay remains
`MissedMaterialOpportunity` for Momentum and `CorrectUncertainty` for
Cycle/Risk. Both existing reward-eligibility outcomes remain
`IneligibleMinimumSamples`, with zero reward and penalty applications.

This pass verifies deterministic reconstruction, one-time reserve consumption,
exact accumulated qualification, additive persistence, replay idempotency, and
the zero-authority boundary. It does not prove model improvement, participant
superiority, a winner, future performance, reward effectiveness, promotion
readiness, Chair learning, or trading readiness.
