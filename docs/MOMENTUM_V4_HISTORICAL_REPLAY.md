# Momentum V4 Historical Replay Lab V1

## Evidence boundary

Soma now has two evidence lanes with different authority.

- Historical replay is fast research and development evidence.
- Live prospective evaluation is slow, authority-grade future evidence.

The existing historical snapshot is classified as
`PreviouslyConsumedResearchEvidence`. It is not an independent holdout and
cannot establish future generalization, participant superiority, reward
eligibility, governance readiness, or trading readiness.

Historical execution is isolated from the live prospective series. It cannot
change prospective event or scorable counts, live participants, parameters,
normalizers, rewards, penalties, voice, tiers, Chair decisions, votes, or
executions. The active committee remains three.

## Canonical dataset discovery

Discovery uses the repository-owned local snapshot decoder and semantic
dataset digest. It validates finalized daily provenance, sanitization,
credential-free read-only origin, symbol and market identity, aggregate
digest, strict OHLCV relationships, finite values, unique increasing
timestamps, and the Upbit daily cadence.

The current canonical selection is:

- provider: `upbit`
- market: `BtcCrypto`
- symbol: `KRW-BTC`
- cadence: `1d`
- rows: 312
- first timestamp: `1757203200000`
- last timestamp: `1784073600000`
- dataset snapshot digest: `91fc9425cd92ce18`
- contamination audit digest: `00321b724d144d4f`

The immutable historical snapshot stores ordered semantic row identities and
source-capsule identities. It does not copy raw rows into a second canonical
dataset.

## Preregistered replay

Both modes freeze their registration before any fold:

- `ProtocolReplay`
- `ExpandingWindowWalkForward`

The V1 walk-forward registration has a 16-row context, horizon one, a minimum
of 64 past labelled training examples, and the
`EveryChronologicallyEligibleEvent` policy. It freezes feature, label,
training, evaluation, interaction-schema, initialization-seed, and participant
identities before execution.

The three research-only fold replicas are:

- `HistoricalRawFeatureLogisticV1`
- `HistoricalRawFeatureInteractionLogisticV1`
- `HistoricalTrainingPrevalenceConstantV1`

Every eligible fold is executed sequentially. For prediction event `t`, the
target is `t + 1`; the latest training event is at most `t - 1`; and the latest
training label is observable by `t`. Feature construction, normalization,
training, and prevalence use the prefix ending at `t` only.

The interaction representation has a fixed order: training-normalized
original features, squares, then pairwise products for every `i < j`.

## Prediction-before-reveal protocol

Each fold has two phases:

1. Build the past-only training set, fit fold-local normalizers, create fresh
   research heads, compute exactly three predictions, persist three seals, and
   persist and reopen the prediction capsule.
2. Only after the capsule reopens successfully, reveal the registered target,
   apply the frozen label policy, and persist the evaluation receipt.

Neutral outcomes are counted but excluded from Brier and binary correctness.
Invalid outcome evidence receives an explicit integrity classification.

All runtime contracts are hand-written protobuf messages. Writes use the
existing create-new temporary file, flush, `sync_all`, decode-and-digest
verification, atomic rename, and final reopen verification path. Runtime
artifacts are ignored and are not committed.

## Public output

Text and JSON expose aggregate research facts only: dataset and registration
identities, fold counts, aggregate metrics, comparison classifications,
chronology and leakage audits, replay identity, runtime duration, and
zero-authority counters.

They do not expose raw OHLCV, per-event predictions or labels, feature values,
parameters, normalizer values, or machine-local artifact locations.

Every aggregate is labelled:

- `HistoricalResearchOnly`
- `NotIndependentHoldout`
- `NotProspectiveAuthority`

No historical winner is selected.

## CLI

```text
--momentum-v4-historical-replay --status --output-format text|json
--momentum-v4-historical-replay --dry-run --output-format text|json
--momentum-v4-historical-replay --execute-local --mode protocol-replay --output-format text|json
--momentum-v4-historical-replay --execute-local --mode expanding-window-walk-forward --output-format text|json
--momentum-v4-historical-backfill-plan --status --output-format text|json
```

Every historical command is offline-only and rejects network permission.

## Backfill and trading boundaries

The backfill plan points only toward data older than the existing snapshot. It
has maximum concurrency one, zero retries, requires explicit future network
authorization, records no known provider request limit, and remains
unexecuted. Sprint 94 performs zero historical network requests.

Trading P&L is `BlockedNoFrozenExecutionPolicy`. A valid simulation requires a
separately preregistered action, sizing, timing, fee, spread, slippage, risk,
and accounting policy. No such policy is inferred from these results.

Official Mamba-3 is not implemented or evaluated by this lab. Chair is
inactive.

## Sprint 95 live-lane isolation

The epoch-two live input and prediction seal completed without reading or
changing historical replay artifacts. Historical fold counts, metrics,
classifications, parameters, normalizers, and replay identity had no influence
on the frozen live roster or predictions. The backfill plan remains
unexecuted.

## Sprint 96 multi-timeframe foundation

The former offline-only backfill plan is superseded by a separately
registered and explicitly authorized multi-timeframe data foundation.
Historical replay semantics in this document remain unchanged.

The new foundation:

- keeps the live epoch-two outcome sealed and pauses epoch three;
- uses native `1m` and `1d` as the only canonical bases;
- derives `3m`, `5m`, `10m`, `1w`, `1mo`, and `1y` without redundant full
  native storage;
- uses bounded native higher-timeframe pages only for semantic comparison;
- proves closed-candle as-of alignment in a no-score protocol replay;
- registers, but does not execute, the next three prediction tasks and A0–A5
  ablation;
- keeps the 70/15/15 historical holdout sealed.

Native month and year samples contain unresolved exact-OHLC mismatches. They
block future multi-timeframe model replay. No model tournament, holdout
opening, live mutation, governance action, or trading simulation is implied.
See `MOMENTUM_MULTITIMEFRAME_HISTORY_V1.md` for the complete contract and
verified aggregate results.
