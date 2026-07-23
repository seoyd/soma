# Momentum V4.2 Future-Prediction Lifecycle

Momentum V4.2 is an additive, two-stage lifecycle for the frozen V4 family. It
separates finalized input-evidence acquisition and prediction sealing from a
later outcome-evidence acquisition and one-time opening. It has no metric,
ranking, winner, reward, active-model, voting, Chair, promotion, or execution
authority.

## Frozen source

The implementation reopens and cross-binds the V4 closure, split,
registration, family, participant identities, V4.1 supplemental registration,
accumulated family, roster, and future-evaluation registration. The frozen
roster remains:

| Participant | Role |
|---|---|
| `RawFeatureLogisticV4` | qualified learned participant |
| `RawFeatureInteractionLogisticV4` | qualified learned participant |
| `TrainingPrevalenceConstantV4` | qualified benchmark |

The reconstruction path verifies configuration, parameter, normalizer,
model-artifact, feature-schema, training-identity, and participant digests. It
uses the training-fitted normalizers and frozen heads. It performs zero
parameter updates, normalizer refits, configuration changes, or participant
identity changes.

## Lifecycle and request budgets

The V4.1 one-request field is classified as `Ambiguous`: the old contract does
not identify whether it applies to input, outcome, or the whole lifecycle.
Before any future transport can be constructed, V4.2 persists and reopens an
additive lifecycle with:

```text
input maximum requests  = 1
input maximum retries   = 0
outcome maximum requests = 1
outcome maximum retries  = 0
maximum concurrency      = 1
```

Prediction must precede outcome access. The outcome stage remains locked after
prediction sealing until the maturity plan's finality boundary and a later
one-time opening.

## Event and context derivation

The event is cadence-aligned and derived from the registered minimum, source
boundary, daily finality, required frozen feature history, protected
timestamps, and request contract. The frozen feature policy needs 9 candles
for one feature row, and the sequence policy needs 8 consecutive feature rows.
The exact prospective context is therefore 16 daily candles ending at the
event timestamp.

The minimum candidate is `1784764800000` (`2026-07-23T00:00:00Z`). Its required
context overlaps protected timestamps. The existing exclusion contracts do not
explicitly permit those timestamps to be reused as inference-only context.
The implementation therefore does not reinterpret them. It derives the first
post-exclusion 16-row candidate:

```text
event timestamp          = 1785974400000
input finality boundary  = 1786060800000
context policy           = ContextUseAmbiguous
readiness                = ContextPolicyAmbiguous
```

The context and input-registration digests are respectively
`74cc98c134216ba5` and `00fde11c360d734f`. The lifecycle digest is
`d4d5416aee82ab26`.

## Input validation and terminal attempts

Transport construction is possible only after the lifecycle, context plan, and
input registration have been persisted, reopened, and matched. Execute mode
also requires explicit network permission and confirmation of the exact
one-time input request.

An input response must have a successful HTTP class, bounded valid JSON,
verified Upbit/BTC/daily identity, the exact registered timestamp set and
count, strict chronology, finalized finite OHLCV, nonnegative volume and trade
value, no duplicate or extra rows, and no outcome timestamp. No provider
health request, ticker request, retry, fallback provider, or manual repair is
allowed. Any attempted input stage is terminal.

Successful input evidence is stored as a bounded raw response, receipt,
verified input capsule, canonical row-identity manifest, and context proof.
The capsule states that outcome rows are absent and labels and metrics were not
accessed.

## Prediction sealing and later maturity

When and only when readiness becomes `ReadyForInputAcquisition`, the same exact
event and context are supplied independently to all three frozen participants.
The interaction participant preserves original normalized features, squared
terms, then pairwise products for every `i < j`. Numeric probability bits exist
only in ignored sealed artifacts. Public output exposes prediction digests,
never numeric probabilities, rows, labels, metrics, parameters, features, or
local paths.

The prediction capsule requires three participant seals, hidden probabilities
and labels, zero outcome access, zero metrics, and no winner. Its append-only
journal records that prediction was sealed before outcome and that the outcome
stage is still locked. The maturity plan contains only future timestamp
identities derived from the frozen horizon. V4.2 never requests the outcome
row.

## Current safe result

The current execution persisted and reopened only the additive lifecycle,
context plan, input registration, and status receipt. Readiness is
`ContextPolicyAmbiguous`, so it constructed zero transport, issued zero network
requests, created no input evidence, performed no participant prediction, and
created no outcome maturity plan. This is a completed safe blocked result, not
fake evidence or a partial prediction.

Status and dry-run are read-only. Execute requires:

```text
--momentum-v4-future-prediction
--execute
--allow-network
--confirm-one-time-future-input-request
--output-format text|json
```

All machine-owned metadata uses manual `prost::Message` contracts and the
existing create-new, flush, sync, reopen, decode, atomic-rename, and final
reopen verification sequence. Repeated execution after a successful seal
returns `PredictionAlreadySealed` with zero new network, model, feature,
prediction, outcome, or write work.

This lifecycle proves exact governance, deterministic planning, frozen-model
reconstruction boundaries, pre-outcome sealing mechanics, and zero authority.
It does not prove a correct prediction, participant superiority, a winner,
model improvement, reward effectiveness, promotion readiness, Chair learning,
or trading readiness.
