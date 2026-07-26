# Momentum Multi-Timeframe Historical Foundation V1

## Scope and authority

This foundation is an isolated research-data and causality lane. It does not
train a live model, score the sealed second live event, change a participant,
select a winner, apply a reward or penalty, invoke the Chair, or authorize
paper or live trading.

The live lane is explicitly paused after the already-sealed epoch two:

- live continuation policy: `PausedAfterSealedEpochTwo`;
- pause digest: `1f6b1750646e9b59`;
- foundation registration: `a7655ab907fb428d`;
- event-two outcome requests and openings: zero;
- epoch three: not registered.

All eight timeframes are research inputs. Their fixed order is:

```text
1m, 3m, 5m, 10m, 1d, 1w, 1mo, 1y
```

Only `1m` and `1d` are canonical raw bases. The `3m`, `5m`, and `10m`
views derive from `1m`; the `1w`, `1mo`, and `1y` views derive from `1d`.
Native higher-timeframe endpoints are bounded cross-check references, not
second canonical stores.

## Registered acquisition

The acquisition plan `e3ec06d69f0b3d56` freezes:

- provider and market: public credential-free Upbit `KRW-BTC`;
- complete one-minute range: 180 UTC days ending before the protected live
  input boundary;
- all provider history older than the frozen 312-row daily snapshot;
- one page each for native `3m`, `5m`, `10m`, `1w`, `1mo`, and `1y`;
- page size 200, concurrency one, zero retries, and a 250 ms minimum delay;
- exact pre-transport request budget 1,400;
- terminal failure per page and explicit resume with verified-page skipping.

The completed acquisition used 1,314 requests: 1,293 minute pages, 15 older
daily pages, and six native reference pages. All 1,314 pages have a verified
receipt and checkpoint; failures and retries are zero.

Upbit documents `to` as an exclusive boundary, a maximum page size of 200,
and omission of intervals without trades. Daily candles use UTC
open/close boundaries. The implementation preserves these semantics rather
than forward-filling missing candles. See the official
[minute candle](https://docs.upbit.com/kr/reference/list-candles-minutes),
[daily candle](https://docs.upbit.com/kr/reference/list-candles-days), and
[rate-limit](https://docs.upbit.com/kr/kr/reference/rate-limits) contracts.

## Canonical storage

Canonical rows are stored in manual-Protobuf chunks of at most 4,096 rows.
Every index binds the ordered chunk chain and aggregate dataset identity.
No raw response body is persisted.

| Base | UTC range | Observed rows | No-trade intervals | Missing evidence | Chunks | Dataset digest | Index digest |
|---|---|---:|---:|---:|---:|---|---|
| `1m` | 2026-01-24 00:00 through 2026-07-22 23:59 | 258,557 | 643 | 0 | 64 | `224c70d04c4eb0ad` | `3695f8b89ef277a7` |
| `1d` | 2017-09-25 through 2026-07-15 | 3,216 | 0 | 0 | 1 | `2b0bc50177b20d6e` | `0d8e28078ba9df30` |

Each successful checkpoint binds its request fingerprint, page receipt, raw
response digest, normalized-row digest, verified-page chunk identity,
exclusive cursor, consumed requests, and remaining registered budget.

## Deterministic views

All interval boundaries are UTC. Fixed minute intervals use open/close
exclusive arithmetic; week, month, and year use calendar boundaries.
Every derived candle binds its ordered base-candle identities and exact OHLCV
aggregation provenance.

| View | Candles | No-trade base intervals | Missing evidence | Index digest |
|---|---:|---:|---:|---|
| `3m` | 86,194 | 25 | 0 | `67e6fd8c5ac5465f` |
| `5m` | 51,717 | 28 | 0 | `83a264399b680fa3` |
| `10m` | 25,860 | 43 | 0 | `18de0f3b7e090c82` |
| `1w` | 459 | 0 | 0 | `850ceefd3ab167d3` |
| `1mo` | 105 | 0 | 0 | `4097b2993e81d0f7` |
| `1y` | 8 | 0 | 0 | `4fe08a915d802456` |

No OHLCV value is fabricated for a no-trade interval. An expected candle
inside verified coverage but absent from the provider response is
`NoTradeInterval`; an expected candle outside verified coverage is
`MissingEvidence`.

## Native cross-check

The numeric tolerance was frozen before acquisition: absolute `1e-12` and
relative `1e-10` for accumulated volume and value only. OHLC values require
exact binary equality after provider parsing.

| Native view | Samples | Exact | Within tolerance | Integrity failure | Model replay allowed | Digest |
|---|---:|---:|---:|---:|---|---|
| `3m` | 200 | 96 | 104 | 0 | yes | `8cd90f73253b2287` |
| `5m` | 200 | 93 | 107 | 0 | yes | `9a081281e05dc0b1` |
| `10m` | 200 | 66 | 134 | 0 | yes | `235972592dd82429` |
| `1w` | 199 | 74 | 125 | 0 | yes | `dc35b7aeceeba303` |
| `1mo` | 105 | 17 | 73 | 15 | no | `1b5a6f5c50c7d0bc` |
| `1y` | 8 | 0 | 4 | 4 | no | `70b5746f20122185` |

The native month and year endpoints expose their period start separately as
`first_day_of_period`; see the official
[month](https://docs.upbit.com/kr/reference/list-candles-months) and
[year](https://docs.upbit.com/kr/kr/reference/list-candles-years) contracts.
The observed month/year OHLC mismatches are not hidden or absorbed by a new
tolerance. They block a future multi-timeframe model replay.

## Causal protocol

Protocol replay `385a3876663ecabf` created 25,856 eligible events at completed
10-minute boundaries. For every persisted event:

1. the latest candle with `close <= prediction timestamp` was selected;
2. every chosen view was closed;
3. future and partial-candle access counts remained zero;
4. a synthetic prediction identity was sealed with zero target access;
5. only then was the registered target timestamp revealed;
6. no target value, metric, score, or performance claim was produced.

The final availability totals are:

```text
1m/3m/5m/10m available = 25,856 each
1d available = 24,992; missing evidence = 864
1w available = 25,424; missing evidence = 432
1mo/1y available = 25,856 each
```

The protocol replay is a causality proof, not a model replay. Therefore the
recorded month/year native mismatch blocks future model execution without
discarding this closed-candle alignment audit.

## Future registrations

The next experiment is registered but not executed:

- three-task registration `99376903a61d45a7`: intraday 10-minute, daily
  one-day, and weekly one-week prediction;
- ordered A0–A5 ablation registration `01d54665e30c494a`;
- sealed 70/15/15 historical holdout `112a88137cf7db2f`;
- holdout begins at `2026-06-25T20:40:00Z`;
- development/validation/holdout event counts: 18,099 / 3,878 / 3,879;
- labels, metrics, and aggregate comparison remain unopened.

The feature-block interface requires independent normalization, exact
dimension checking, provenance binding, private numeric values, and complete
context. Learned cross-timeframe fusion is intentionally not implemented.

## CLI and persistence

```text
--momentum-mtf-history --status --output-format text|json
--momentum-mtf-history --dry-run --output-format text|json
--momentum-mtf-history --register-foundation --execute-local
--momentum-mtf-history --execute-backfill --allow-network --confirm-bounded-mtf-history-backfill
--momentum-mtf-history --derive-views --execute-local
--momentum-mtf-history --protocol-replay --execute-local
```

Registration, network backfill, derivation, and protocol replay are distinct
modes. Historical flags cannot authorize the live outcome stage.

Every runtime contract uses the existing manual-Protobuf builder/reader and
atomic writer: create-new temporary file, flush, `sync_all`, reopen, decode,
semantic-digest validation, atomic rename, and final reopen validation.
Runtime evidence remains ignored and uncommitted.

## Proof boundary

Proven:

- bounded sequential acquisition and resumable page evidence;
- two canonical bases and six deterministic derived views;
- exact no-trade versus missing-evidence classification;
- closed-candle-only as-of joins and prediction-before-reveal ordering;
- sealed future registrations and unopened holdout;
- unchanged live artifacts and roster with zero live authority.

Not proven:

- multi-timeframe model improvement or future generalization;
- independent holdout performance;
- native monthly or yearly aggregation equivalence;
- participant superiority, winner, reward effectiveness, or Chair learning;
- paper- or live-trading readiness.
