# Momentum V4.3 Protected Inference Context

Momentum V4.3 adds one narrow authorization to the frozen V4/V4.1 family:
protected timestamps may supply canonical raw daily OHLCV only as read-only
feature context for a strictly later prospective event. The authorization does
not weaken the original exclusions and does not change any protected artifact.

## Authorization boundary

Authorization is valid only when the three roster identities, parameters,
normalizers, model artifacts, feature schemas, and training identities reopen
unchanged. The model source boundary must be strictly earlier than every
protected timestamp. The prospective event and its horizon-one outcome must
both be outside the protected set.

The only permitted class is `RawOhlcvInferenceContext`. All of the following
remain forbidden:

- parameter training or updates;
- normalizer fitting or refitting;
- label construction or opening;
- qualification;
- metric, ranking, or winner computation;
- reward or penalty use;
- event- or outcome-timestamp selection;
- reuse of a prior outcome capsule, opening bundle, attribution, or reward
  artifact as an input-value source.

Raw values may come only from an independently stored canonical raw provider
row or the newly registered public input request. Provenance remains separate
even if values happen to match an older outcome record.

## Append-only usage ledger

Every accepted input row receives one of four explicit classifications:

| Class | Meaning |
|---|---|
| `ExistingFrozenHistoricalContext` | existing canonical source context |
| `ProtectedRawInferenceContext` | protected raw OHLCV used only for features |
| `NewIncrementalInferenceContext` | newly acquired non-protected context |
| `ProspectiveEventInput` | the prospective event candle |

Each entry binds the timestamp and canonical raw-row digest. Feature
construction is true; training, normalizer fitting, label, metric, and reward
uses are false. The ledger is stored only after an exact successful input
response, so pre-finality execution creates no placeholder ledger.

## Supersession and corrected event

The immutable V4.2 plan and registration remain in place, but a V4.3
supersession makes the old executor permanently return
`SupersededInputRegistration` with zero transport. Supersession is legal only
because the prior attempt count is zero and no prior input receipt, input
capsule, or prediction capsule exists.

With protected raw inference context explicitly authorized, the first event is
derived from the original registered minimum:

```text
event                  2026-07-23T00:00:00Z
16-row context start   2026-07-08T00:00:00Z
input finality         2026-07-24T00:00:00Z
outcome timestamp      2026-07-24T00:00:00Z
outcome finality       2026-07-25T00:00:00Z
```

Production code derives these boundaries from persisted policy identities and
daily cadence; they are not selected from provider results.

## Request and prediction safety

The corrected input registration requires exactly 16 finalized daily
`KRW-BTC` rows in one credential-free, read-only request with concurrency one
and zero retries. Before input finality, execute mode persists and reopens only
the authorization, supersession, corrected context plan, corrected input
registration, and safe status receipt.

After actual finality, a successful exact response may reconstruct the same
three frozen participants and seal one hidden probability per participant.
There are zero parameter updates and normalizer refits. Prediction is sealed
before any outcome access. The later outcome plan remains locked and Sprint 83
does not request, read, label, score, rank, reward, promote, vote, execute, or
open that outcome.

All machine artifacts use manual `prost::Message` envelopes and the existing
atomic create, flush, sync, reopen, semantic verification, rename, and final
reopen sequence. Public status exposes identities and zero-authority counters,
not probabilities, OHLCV, labels, metrics, parameters, features, returns, or
local paths.
