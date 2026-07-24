# Momentum V4.4 Future Outcome and Opening

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
evaluation policies.

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
participant evaluations, and zero winner, reward, or penalty. A distinct
append-only V4.4 ledger records one event without merging any prior prospective
experiment.

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
All V4.4 machine artifacts use manual `prost::Message` contracts and verified
create-new temporary write, flush, sync, reopen/decode, atomic rename, and
final reopen/decode storage.

## Current verified state

At `2026-07-24T03:43:12Z`, finality had not arrived. Status, dry-run, and
execute agreed on `AwaitingOutcomeFinality`, registration
`52da8d28f246ee4b`, and request fingerprint `ccf0335feac46846`. Runtime storage
contains only the V4.4 registration and safe status. There is no outcome
attempt, HTTP result, outcome receipt, outcome capsule, opening authorization,
opening receipt, evaluation ledger, metric, ranking, winner, reward, or
authority action.

This verifies the implementation and the pre-finality fail-closed state. It
does not establish prediction correctness, model improvement, participant
superiority, a winner, reward effectiveness, promotion readiness, Chair
learning, or trading readiness.
