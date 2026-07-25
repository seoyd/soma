# Momentum V4 Prospective Series

## Purpose

The V4 prospective series is an append-only continuation of the first sealed,
opened, and evaluated Raw-feature V4 event. It adopts that event without
rewriting it, preserves the identical frozen three-participant roster, and
derives each later candidate from the persisted daily cadence and actual
registration time.

The current completed prospective-event count is one. The current scorable
event count is one, and reward eligibility remains
`IneligibleMinimumSamples`. No winner, ranking, reward, penalty, Chair action,
or trading action exists.

## Current registered state

Epoch two is preregistered as event timestamp `1784937600000`. Its input
finality is `1785024000000`, its horizon-one outcome timestamp is
`1785024000000`, and outcome finality is `1785110400000`. Readiness is
`RegisteredAwaitingInputFinality`.

The exact 16-row context runs daily from `1783641600000` through
`1784937600000`. Fifteen canonical rows are reused; the exact missing set is
`[1784937600000]`. The adjacent candidate disposition is
`SkippedPriorOutcomeAlreadyOpened`. Registration and status replay perform
zero network, raw-read, reconstruction, feature, prediction, outcome,
authority, and write work.

No input receipt, input capsule, context-assembly proof, prediction seal,
prediction capsule, series journal entry, or outcome plan exists yet.

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
outcome access. The horizon-one outcome plan remains locked, with zero outcome
requests and openings. If execution is interrupted after a successful input
receipt, replay reopens the digest-bound raw evidence and resumes prediction
without another network request. Completed and terminal replay paths perform
zero new work.

## Manual interface

Status:

```text
--momentum-v4-prospective-series
--status
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

No command registers and executes multiple epochs. There is no daemon,
scheduler, background loop, automatic daily request, or automatic outcome
opening.

## Persistence and scope

All fourteen series artifacts use manual `prost::Message` contracts and the
existing verified create-new temporary write, flush, sync, reopen/decode,
atomic rename, and final reopen/decode sequence. Runtime evidence is
append-only and separate from the first event.

The next step is to preregister and seal a second event using the identical
frozen roster. Official Mamba-3 is not implemented or evaluated. Chair
functionality remains inactive.
