# Momentum Raw-Feature Learned Path V4

Momentum raw-feature V4 is an offline, additive research path over the exact
V1, V2, and V3 Momentum history. It preserves every earlier artifact and closes
the current frozen-Mamba path only for the current encoder, 312-row evidence,
feature policy, and label policy. It does not remove Mamba from Soma or make a
claim about other Mamba encoders, evidence, or policies.

## Scope-limited frozen-Mamba closure

Execution reopens the exact V1, V2, and V3 families, the V3 split, and the V3
route decision. Closure succeeds only when the four V3 learned routes remain
rejected, the genuine-Mamba qualified count is zero, the route decision remains
`AllRepresentationRoutesCollapsed`, and no V3 roster or evaluation registration
exists.

The verified closure is `ClosedForCurrentEvidenceAndPolicy`, digest
`2d7667c8cf72ed5e`. Head-only repair, another frozen-representation sweep, and
use of the closed encoder as a V4 parent are forbidden. Reopening requires new
encoder, evidence, and preregistration identities.

The historical trainer capability is retained as
`MomentumFrozenMambaLegacy/terminal-current-evidence-policy`. V4 records the
separate `MomentumRawFeatureV4/ShadowOnly` research capability without changing
the active capability registry or canonical committee state.

## Fresh V4 split and preregistration

The V4 split is derived from the persisted V3 split rather than fixed observed
indices. The current verified result is:

```text
training         [0, 240)
purge            [240, 264)
fresh validation [264, 288)
final reserve    [288, 312)
```

The entire V3 validation range becomes the V4 purge. The first validation-sized
block of the V3 final reserve becomes V4 validation, and the remaining block
stays untouched. The split digest is `a072cecd98df1f44`. Prior qualification,
prospective, historical-test, and future-evaluation overlap counts are zero.
No row or label in `[288,312)` is built or read.

Registration `b6ce06fd9f226277` is persisted and reopened before validation
inference. It freezes exactly two learned configurations and one benchmark:

- `RawFeatureLogisticV4`
- `RawFeatureInteractionLogisticV4`
- `TrainingPrevalenceConstantV4`

The two learned participants use fresh deterministic heads, bounded registered
training policies, and training-only normalizers. They reuse no V1, V2, or V3
parameters, normalizers, or predictions. No second configuration batch or
result-selected feature set exists.

## Raw and interaction features

The raw logistic participant consumes the existing ordered engineered Momentum
feature vector directly. The interaction participant expands the same
training-normalized vector in deterministic order: original features, squared
terms, then every pairwise product for `i < j`. Cubic terms, random projections,
learned feature selection, and validation-selected interactions are absent.

The interaction expansion has a schema-bound dimension and rejects nonfinite
input, inconsistent width, and nonfinite output. An independent normalizer is
fit on the expanded V4 training rows only. Both learned heads use the existing
Brier-loss SGD and finite gradient and parameter guards. Validation parameter
updates are zero.

The constant benchmark derives its probability only from V4 training labels.
Zero probability variance is valid for its benchmark role and is not treated as
learned participation.

## Verified qualification result

The current validation index block is 24 rows. The additive validation-yield
audit derives 23 valid labelled samples and one neutral exclusion from the
frozen policy, with zero horizon-unavailable and feature-unavailable indices.
The required minimum remains 24, so substantive qualification was not
possible. Audit digest: `62069972025760a4`. Accordingly, the immutable receipt
statuses are:

| Participant | Status |
|---|---|
| `RawFeatureLogisticV4` | `RejectedInsufficientValidation` |
| `RawFeatureInteractionLogisticV4` | `RejectedInsufficientValidation` |
| `TrainingPrevalenceConstantV4` | `RejectedInsufficientValidation` |

The interaction block-zero audit is deterministic and reports
`MaterialInteractionContribution`. This describes parameter contribution only;
it does not override insufficient validation and is not evidence of nonlinear
progress or model improvement.

Family `bca7665e1e1b2012` retains all three participants, all three receipts, and
the contribution audit. The qualified learned and benchmark counts are both
zero. No winner was selected, and the family is ineligible for active committee
use, promotion, and reward.

The corrected path decision is `InsufficientFreshValidation`, digest
`bd34ab49577e8919`. The future roster and evaluation statuses are both
`QualificationEvidenceInsufficient`; no roster or evaluation registration
exists, and no minimum future timestamp is assigned. This correction does not
alter the three original receipts.

## Persistence, CLI, and safety

Closure, split, registration, validation-yield audit, participants, receipts,
contribution audit, family, decision, optional roster, optional evaluation
registration, and journal use hand-written `prost::Message` contracts and the
existing verified atomic writer. Semantic hardening added the audit, corrected
decision, and corrected journal without overwriting the original 13 sidecars.
The directory contains 16 sidecars, of which 14 form the current applicable
identity set; an identical rerun writes zero and duplicate-rejects all 14.

The offline command is:

```text
--momentum-raw-feature-v4 --status|--dry-run|--execute-local
--output-format text|json
```

Status and dry-run write nothing. Every mode rejects network and authority
flags. Public output contains only statuses, digests, participant IDs, counts,
the optional future boundary, reward-eligibility replay, and zero safety
counters. Rows, features, expansion values, probabilities, labels, metrics,
parameters, gradients, and local paths remain private.

Network, transport, credential, prospective, historical-test, future-evaluation,
final-reserve, active-model, Chair, vote, reward, penalty, voice, cooldown,
promotion, quarantine, and execution counts are zero. The active committee
count remains three. Cycle/Risk remains `ProviderContractUnverified` and
Value/Quality remains `TrainerUnavailable`. Persisted attribution remains
`MissedMaterialOpportunity` for Momentum and `CorrectUncertainty` for
Cycle/Risk; both reward-eligibility results remain `IneligibleMinimumSamples`
with zero applications.

This result proves the bounded V4 protocol and an underpowered initial
qualification attempt. It does not prove that a participant substantively
failed, model improvement, participant superiority, a winner, future
performance, reward effectiveness, promotion readiness, Chair learning, or
trading readiness.
