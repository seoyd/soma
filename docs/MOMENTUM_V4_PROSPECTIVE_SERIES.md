# Momentum V4 Prospective Series

## Purpose

The V4 prospective series is an append-only continuation of the first sealed,
opened, and evaluated Raw-feature V4 event. It adopts that event without
rewriting it, preserves the identical frozen three-participant roster, and
derives each later candidate from the persisted daily cadence and actual
registration time.

The current completed prospective-event count is two. Both events are
scorable, and reward eligibility remains `IneligibleMinimumSamples`. No
winner, ranking, reward, penalty, Chair action, or trading action exists.

## Input-sealed state

Epoch two is preregistered as event timestamp `1784937600000`. Its input
finality is `1785024000000`, its horizon-one outcome timestamp is
`1785024000000`, and outcome finality is `1785110400000`.

The exact registered input executed once inside the legal window. It consumed
one request with zero retries, accepted the single missing finalized row, and
sealed the unchanged three-participant live roster. Input-stage readiness was
`PredictionAlreadySealed`.

The exact 16-row context runs daily from `1783641600000` through
`1784937600000`. Fifteen canonical rows are reused; the exact missing set is
`[1784937600000]`. The adjacent candidate disposition is
`SkippedPriorOutcomeAlreadyOpened`. Registration and status replay perform
zero network, raw-read, reconstruction, feature, prediction, outcome,
authority, and write work.

The successful input receipt is `ec2806f2d5d234e5`, the input capsule is
`3a918381cf1cedfa`, and the context-assembly proof is `41aa5585171d28a5`.
Exactly three private seals were bound by prediction capsule
`f0fc2d24e1c920e4`, journal `ed46f8a8b3f4f806`, and locked outcome plan
`ae798b355d36bb74` before outcome access. Numeric probabilities remain
private.

## Completed event-two outcome

At `2026-07-27T03:58:12Z`, actual UTC was after the registered outcome
finality boundary. Two text statuses, two JSON statuses, and text and JSON
dry-runs agreed on `ReadyForOutcomeAcquisition`, the sealed event-two chain,
the exact one-row request preview, and zero work.

The public Upbit outcome request executed exactly once with zero retries and
maximum concurrency one. The successful receipt is `738d106d50a89df6`; the
still-closed outcome capsule is `c67ab84dffddf3eb`. Acquisition stopped at
`ReadyForOutcomeOpening` with zero label derivations, private prediction
reads, evaluations, or opening attempts.

A separate network-free local authorization then opened only event two. The
authorization is `e095a9064ad9711a`, the opening bundle is
`6c7e1d55a1da912b`, and the event-two ledger entry is
`9a8711351dbcb931`. The frozen policy classified the event
`ScorableBinaryOutcome`; all three live participants have public evaluation
status `Scored`. No numeric prediction, score, correctness value, return, or
OHLCV was published.

The complete append-only ledger now derives two completed and two scorable
events. Eligibility receipt `c94ac1fe15dcfb0c` remains
`IneligibleMinimumSamples`. Additive pause `e616e1deb5e935c9` records
`PausedAfterCompletedEpochTwo`; epoch three remains absent.

## Immutable series contract

The series contract binds:

- the frozen roster and participant identities;
- parameter, normalizer, model, feature, label, evaluation, and minimum-sample
  policy identities;
- the existing event-one prediction, opening, ledger, and eligibility chain;
- one-day cadence, 16-row feature context, and horizon one;
- at most one open epoch;
- one input request, zero retries, and concurrency one;
- explicit manual network permission;
- zero training, refitting, selection, governance, reward, or execution
  authority.

Event-one adoption is additive. Its already opened result cannot influence
participant membership, event continuation, or the next event timestamp.

## Candidate and epoch derivation

The adjacent cadence candidate is audited separately. A candidate that was
already consumed as the prior event's outcome, or whose input-finality
boundary passed before registration, is recorded as skipped without counting
it as a model failure or creating reward or penalty consequences.

The next legal event is the first later cadence-aligned candidate that is not
the prior opened outcome timestamp and can still be registered before input
finality. Production does not hardcode its timestamp, readiness, missing-row
count, or disposition.

Registration persists and reopens the series contract, event-one adoption,
candidate-gap audit, canonical context-delta plan, epoch registration, and
safe status. It rejects network permission and does not create an input
receipt, input capsule, prediction, journal entry, or outcome plan.

## Canonical context and one-time input

The Data Broker reconstructs the exact 16 daily context timestamp identities
from the registered event. Existing canonical raw rows are reused by verified
row and source-capsule digests. The prior opened outcome row may be reused only
as raw inference context; an opening bundle, label, score, correctness value,
or reward state can never supply feature values.

Only the exact contiguous missing timestamp set is requestable. Existing
canonical rows are never refetched. Before input finality, execution performs
zero transport. At or after outcome finality, an unsealed epoch is expired and
also performs zero transport.

Inside the legal window, input execution requires the exact epoch, network
permission, and one-time confirmation. It constructs at most one
credential-free, read-only request with no retry. The raw response and
normalized response must agree and contain exactly the registered finalized
rows with valid chronology and OHLCV shape. Wrong, duplicate, missing, extra,
unfinished, or outcome rows fail terminally.

Transport and validation failures persist one terminal receipt and report
`PriorInputAttemptTerminal`. Replay returns that state before transport
construction, so a consumed request can never be retried or replaced.
A terminal receipt carrying successful response identity or a mismatched
registration binding remains an `IntegrityFailure`.

Status and dry-run both expose the same sanitized registered request preview:
registration time, canonical reused timestamps, provider, market, cadence,
request boundaries, request fingerprint, prior-attempt count, and receipt and
prediction presence. Dry-run cannot accept network authority, construct a
transport, or write an artifact.

## Prediction sealing and recovery

After exact input verification, the same assembled 16-row context is supplied
to all three frozen participants. Reconstruction verifies all frozen
identities and performs zero parameter updates or normalizer refits. Exactly
three private prediction seals are required.

Public status exposes prediction digests but never numeric probabilities,
OHLCV, returns, labels, scores, correctness, parameters, normalizer values,
engineered features, or private evaluation ordering. The prediction capsule
keeps probabilities and labels closed and records zero outcome, metric,
winner, and ranking work.

The append-only journal proves that deterministic registration preceded input
finality, input acquisition preceded prediction, and prediction preceded
outcome access. It binds the event-one adoption, exact context-delta plan, and
three participant-seal identities. Each seal directly binds epoch two and the
context-use proof. The input capsule directly binds the delta plan, provider,
and consumed one-attempt budget. The horizon-one outcome plan remains locked,
with zero outcome requests and openings.

If execution is interrupted after a successful input receipt, replay reopens
the digest-bound raw evidence and resumes prediction without another network
request only while the prediction window remains open. At or after outcome
finality, recovery returns `PredictionSealWindowExpired` before raw loading,
participant reconstruction, prediction, or writes. Completed and terminal
replay paths perform zero new work.

Status and dry-run never perform that local recovery. Before outcome finality
they report `ReadyForLocalPredictionRecovery` with zero raw loading,
reconstruction, prediction, transport, or writes; only an explicitly
authorized input execution replay may resume the local seal.

Completed-chain validation binds each reopened seal by the seal and prediction
digests frozen in the capsule. Filesystem directory order has no semantic
authority. Status, dry-run, and repeated execute-input replay all return the
existing sealed chain with zero network, raw loading, reconstruction,
prediction, or writes.

## Manual interface

Status:

```text
--momentum-v4-prospective-series
--status
--output-format text|json
```

Dry-run:

```text
--momentum-v4-prospective-series
--dry-run
--output-format text|json
```

Preregister the next legal epoch:

```text
--momentum-v4-prospective-series
--register-next-epoch
--execute-local
--output-format text|json
```

Execute one registered input:

```text
--momentum-v4-prospective-series
--epoch <n>
--execute-input
--allow-network
--confirm-one-time-prospective-input-request
--output-format text|json
```

Acquire the registered event-two outcome:

```text
--momentum-v4-prospective-series
--epoch 2
--execute-outcome
--allow-network
--confirm-one-time-prospective-outcome-request
--output-format text|json
```

Open the acquired outcome locally:

```text
--momentum-v4-prospective-series
--epoch 2
--open-outcome
--execute-local
--confirm-one-time-prospective-outcome-opening
--output-format text|json
```

No command registers and executes multiple epochs. There is no daemon,
scheduler, background loop, automatic daily request, or automatic outcome
opening.

## Persistence and scope

The series and event-two artifacts use manual `prost::Message` contracts and
the existing verified create-new temporary write, flush, sync, reopen/decode,
atomic rename, and final reopen/decode sequence. Runtime evidence is
append-only and separate from the first event.

Completed status, dry-run, acquisition replay, and opening replay all return
`OutcomeAlreadyOpened` before transport construction, raw loading, private
prediction access, label derivation, evaluation, ledger append, or writes.
Live continuation is paused. The next research priority is micro
feature/label challenger design in the historical research lane. Official
Mamba-3 is not implemented or evaluated, and Chair functionality remains
inactive.

## Bounded historical continuation

Live events one and two remain complete, continuation remains paused after
completed epoch two, and no epoch three exists. Q1 and Q2 require feature and
label redesign.

The separate historical lane completed label and feature diagnostics and
registered compact micro challengers. It trained no challenger model and
opened no historical holdout.
