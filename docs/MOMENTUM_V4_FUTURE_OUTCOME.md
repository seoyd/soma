# Momentum V4.4 Future Outcome and Opening

## Prospective-series continuation boundary

The first V4 outcome and opening remain immutable inputs to an additive
prospective-series contract. Event-one counts remain one total and one
scorable event, with reward eligibility `IneligibleMinimumSamples`. Its raw
outcome row may be referenced as context-only evidence for a later registered
event, but its opening bundle, label, participant evaluations, scores, and
eligibility state cannot supply features or affect cadence continuation.

The second registered event had a separately locked horizon-one outcome plan.
Its prospective input stage performed zero outcome requests, openings, label
access, metrics, winner selection, ranking, reward, or penalty work. Outcome
acquisition and opening later ran as two explicitly authorized stages.
Official Mamba-3 is not implemented or evaluated, and Chair functionality
remains inactive. See `MOMENTUM_V4_PROSPECTIVE_SERIES.md` for the append-only
series and manual contracts.

## Boundary

V4.4 continues the single sealed V4.3 prospective event. It does not retrain,
refit, replace, rank, promote, reward, penalize, vote, trade, or mutate an
active model. The frozen roster remains two learned participants and one
training-prevalence benchmark.

Before any outcome transport, the implementation reopens and validates the
complete V4.3 chain: lifecycle, protected-context authorization, supersession,
corrected context plan, input registration, successful input receipt, input
capsule, context verification, context-use ledger, three participant seals,
prediction capsule, prediction journal, and outcome plan. Every cross-reference
and semantic digest is verified. Missing, duplicate, malformed, or changed
artifacts fail closed before transport.

## Outcome acquisition

The V4.4 registration is derived only from the sealed outcome plan. Its exact
contract is one credential-free, read-only Upbit daily request for one
`KRW-BTC` candle:

```text
outcome timestamp = 2026-07-24T00:00:00Z
request to         = 2026-07-25T00:00:00Z
maximum requests   = 1
maximum concurrency = 1
maximum retries    = 0
```

Before `2026-07-25T00:00:00Z`, execute persists and reopens only the
registration and safe status. It returns `AwaitingOutcomeFinality` with zero
request attempts, transport constructions, retries, row reads, label reads,
metrics, receipts, capsules, and opening artifacts. It does not sleep or poll.

At or after finality, execute requires both network permission and the exact
one-time request confirmation. The request is reconstructed from the persisted
registration. There are no health, server-time, market-list, ticker,
pagination, fallback, participant-specific, or context requests.

An accepted response must be bounded valid JSON and match the normalized
provider response exactly. It must contain one finalized daily `KRW-BTC` row
at the registered timestamp, with finite valid OHLCV, nonnegative volume and
trade value, no duplicate, missing, prior, or extra timestamp, and
credential-free read-only provenance. A transport, HTTP, or validation failure
creates one terminal receipt and cannot retry. Exact success stores the raw
response, row-identity proof, closed outcome capsule, and receipt using the
existing verified atomic persistence path.

Acquisition never opens labels or probabilities and never computes a metric or
winner. Once a successful capsule exists, repeated execute returns
`OutcomeEvidenceAcquired` before constructing a transport or writing anything.

## Separate local opening

Opening is a different command and rejects network permission. Status and
dry-run do not convert sealed probability bits to numeric values, do not read
private outcome values, do not compute labels or metrics, and do not write
opening artifacts.

Execute-local requires the exact one-time owner confirmation. Before private
numeric access, it persists and reopens an authorization binding the outcome
registration, receipt and capsule, prediction capsule and journal, all three
participant seals and prediction digests, and the frozen feature, label, and
evaluation policies. The authorization also binds explicit prohibitions on
winner selection, ranking, reward and penalty application, Chair action, voice
mutation, promotion, and trading.

The frozen horizon-one label policy is reused without a new threshold. Returns
strictly outside the frozen dead zone produce a binary scorable event. A
return inside the dead zone remains `NeutralOutcomeExcluded` when neutral
labels are disabled.

All three sealed probabilities and the one label are opened in one local
operation. A scorable event creates exactly three private single-event Brier
and correctness contributions. A neutral event creates three excluded
evaluations with no score computation. Public status exposes only status names
and digests; it never exposes OHLCV, returns, labels, probabilities, Brier
values, correctness values, parameters, features, or local paths.

The opening bundle records one attempt, one opened event, exactly three
participant evaluations, and zero winner, ranking, reward, penalty, or Chair
action. The sealed outcome capsule binds zero reward and penalty application,
and the distinct append-only V4.4 ledger repeats those zero-application
bindings without merging any prior prospective experiment.

Reward eligibility is recomputed from the new ledger, preserved participant
roles, event counts, the existing minimum-sample gate, and integrity state.
The derived result may be an ineligible state or an eligible candidate, but
reward, penalty, voice, cooldown, promotion, and quarantine application counts
remain zero. Repeated opening returns `AlreadyOpened` before label access,
prediction-value access, metric work, ledger append, eligibility recomputation,
or writes.

## CLI

Outcome acquisition:

```text
--momentum-v4-future-outcome
--status|--dry-run|--execute
--allow-network
--confirm-one-time-future-outcome-request
--output-format text|json
```

Local opening:

```text
--momentum-v4-future-outcome-opening
--status|--dry-run|--execute-local
--confirm-one-time-future-outcome-opening
--output-format text|json
```

The acquisition command rejects opening and unrelated authority flags. The
opening command rejects network, acquisition, and unrelated authority flags.
Acquisition status and dry-run expose the complete registered preflight
contract in both text and JSON, including the prediction, journal, plan,
provider, market, cadence, timestamp, row-count, request-budget, and zero-work
fields. Opening status and dry-run likewise expose write and sealed-prediction
read counts without exposing private values.

All V4.4 machine artifacts use manual `prost::Message` contracts and verified
create-new temporary write, flush, sync, reopen/decode, atomic rename, and
final reopen/decode storage.

## Current verified state

PR #19 was merged into `main` as
`98347586cccbed1809bd677aa2bd43e6eb7e25b1`. At
`2026-07-25T00:13:58Z`, actual UTC was after the persisted finality boundary.
Two text statuses, two JSON statuses, and text and JSON dry-runs agreed on
`ReadyForOutcomeAcquisition`, registration `52da8d28f246ee4b`, request
fingerprint `ccf0335feac46846`, the complete registered request contract, and
zero preflight work.

The registered public request executed exactly once. The response passed the
HTTP, provider, market, cadence, exact single-row, finality, bounded JSON, and
OHLCV-shape checks. Receipt `1cba98966e0b6002` and sealed capsule
`e1b3f829d186b3b3` were persisted with labels, probabilities, metrics, winner,
reward, and penalty remaining closed. Confirmed acquisition replay returned
the existing state with zero transport, reads, metrics, or writes.

Opening status and dry-run agreed on a ready sealed state and zero private
reads or writes. The first execute-local attempt exposed a correctness defect
before authorization persistence: the lifecycle data-access policy identity
was incorrectly compared with the frozen sequence label policy. The corrected
implementation binds the actual horizon, dead-zone, and neutral-handling
policy directly and rejects any substituted policy digest.

After the correction, the one local opening completed atomically. The event
was classified `ScorableBinaryOutcome`; all three participant evaluations have
public status `Scored`; total and scorable V4 event counts are one; and reward
eligibility is `IneligibleMinimumSamples`. No private OHLCV, return, label,
probability, score, or correctness value was published. Opening replay returned
`AlreadyOpened` with zero new reads, metrics, ledger work, eligibility work, or
writes.

The original 147-artifact aggregate identity remains
`6f7a560954e1b4e5dfef87b40f88126d1662fab580c34271fa33efd56e014239`.
Winner, ranking, reward, penalty, voice, tier, cooldown, promotion, quarantine,
Chair, vote, active-model, paper, and live-execution actions remain zero.

This verifies the first immutable V4 prospective event and its fail-closed
authority boundaries. It does not establish model improvement, participant
superiority, a winner, reward effectiveness, promotion readiness, Chair
learning, official Mamba-3 behavior, or trading readiness.

## Event-two prospective-series close

Event two was already sealed before its outcome became accessible. At
`2026-07-27T03:58:12Z`, after its registered finality boundary, six read-only
preflights agreed on `ReadyForOutcomeAcquisition`, the exact one-row request,
and zero work.

The registered request executed once, returned one finalized row at the exact
timestamp, and used zero retries. Successful receipt `738d106d50a89df6` and
closed outcome capsule `c67ab84dffddf3eb` were persisted before any label or
private prediction access. Acquisition then reported
`ReadyForOutcomeOpening`.

Separate local authorization `e095a9064ad9711a` opened only the sealed
event-two outcome and predictions. Opening bundle `6c7e1d55a1da912b`
classified the result `ScorableBinaryOutcome`; each of the three live
participants has public status `Scored`. Event-two ledger entry
`9a8711351dbcb931` derives two completed and two scorable live events.
Eligibility remains `IneligibleMinimumSamples`.

No winner, ranking, reward, penalty, Chair action, vote, paper execution, live
execution, or trade occurred. Additive completed pause
`e616e1deb5e935c9` preserves the prior pause and records that epoch three is
absent. Completed status and dry-run replays return `OutcomeAlreadyOpened`
with all work counters zero. The historical holdout remains closed.
