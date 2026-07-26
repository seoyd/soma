# Momentum Qualified-Six Replay V1

## Scope

This is a separate offline historical-research experiment over the six
semantically qualified views:

```text
included: 1m, 3m, 5m, 10m, 1d, 1w
excluded unresolved: 1mo, 1y
task: next completed 10m close direction
```

It does not alter or weaken the blocked full-eight registration. The 15
monthly and four yearly forensic failures remain unresolved under the
unchanged absolute `1e-12` and relative `1e-10` accumulation tolerances.
OHLC equality remains exact.

The replay is development and validation evidence only. It cannot open the
sealed holdout, read a live outcome, change a live participant or parameter,
select a winner, create a ranking, apply reward or penalty, invoke the Chair,
or execute paper or live trading.

## Registration and chronology

Registration `e54c65c8ecbfef85` binds:

- a common six-view eligible range of 25,841 completed 10-minute events;
- a chronological 70/15/15 development, validation, and sealed-holdout split;
- a 512-example minimum derived as the next power of two above ten times the
  largest participant input dimension;
- a fixed 4,096-example maximum training window;
- fresh daily UTC refits using only targets strictly before the day boundary;
- independent per-view normalizers fitted only from that day's training rows;
- no within-day refit, live parameter load, or prior-fold parameter reuse.

Every event uses the latest 16 closed candles in each required view. The
prediction timestamp is a completed 10-minute boundary and the target is the
next completed 10-minute candle. Before prediction, each day's six private
normalizer receipts and five private participant receipts are atomically
persisted, reopened, decoded, and reconstructed. All five predictions are
then sealed and the daily prediction capsule is likewise persisted and
reopened before any target value is read.

## Participants

| ID | Input |
|---|---|
| Q0 | training-label prevalence constant |
| Q1 | `10m` |
| Q2 | `1m`, `3m`, `5m`, `10m` |
| Q3 | `1d`, `1w` |
| Q4 | all six qualified views |

All learned participants are fresh deterministic logistic research models.
There are no interaction features and no result-selected participant.

## Verified replay result

| Partition | Events | Training-only | Predictions | Scorable | Neutral | Invalid | Daily refits |
|---|---:|---:|---:|---:|---:|---:|---:|
| Development | 18,088 | 560 | 17,528 | 17,395 | 133 | 0 | 122 |
| Validation | 3,876 | 0 | 3,876 | 3,846 | 30 | 0 | 28 |

Mean Brier score and binary correctness are aggregate, paired, historical
research metrics:

| Participant | Development Brier | Development correctness | Validation Brier | Validation correctness |
|---|---:|---:|---:|---:|
| Q0 | 0.250078 | 0.498592 | 0.249906 | 0.518201 |
| Q1 | 0.249804 | 0.516183 | 0.249939 | 0.524181 |
| Q2 | 0.250211 | 0.515953 | 0.249629 | 0.535881 |
| Q3 | 0.251264 | 0.500259 | 0.250299 | 0.513521 |
| Q4 | 0.251110 | 0.514573 | 0.250305 | 0.518981 |

The paired Q0 comparisons classify Q1 and Q2 as mixed across partitions.
Q3 and Q4 have higher Brier than Q0 across both partitions. The contribution
comparisons classify Q2 versus Q1 as mixed, Q4 versus Q2 as higher Brier
with the added blocks, and Q4 versus Q3 as mixed. Q0's validation
probabilities meet the registered collapse threshold; the four learned
participants do not.

These classifications do not select a winner and do not establish future,
independent, prospective, paper-trading, or live-trading performance.

## Closed holdout and authority audit

The holdout remains sealed. Holdout label reads, metric computations, and
participant predictions are all zero. Month/year loads,
future access, partial-candle access, unqualified-view access, live outcome
requests/openings, live changes, winner/ranking actions, reward/penalty
applications, Chair decisions, votes, voice/tier changes, cooldowns,
promotions, quarantines, historical-participant speaking rights and committee
memberships, executions, network attempts, transport constructions, and
credential reads are all zero.

The protected live artifact tree and active roster are unchanged. The
already-sealed second live event remains sealed and epoch three remains
unregistered.

## CLI and persistence

```text
--momentum-mtf-qualified-six-replay --status --output-format text|json
--momentum-mtf-qualified-six-replay --dry-run --output-format text|json
--momentum-mtf-qualified-six-replay --register --execute-local --output-format json
--momentum-mtf-qualified-six-replay --execute-local --partition development --output-format json
--momentum-mtf-qualified-six-replay --execute-local --partition validation --output-format json
```

There is deliberately no holdout execution mode. Execution modes reject
network and unrelated authority flags. A preregistered minimum larger than
available development support returns `InsufficientTrainingSupport`;
conflicting persisted identities return `IntegrityFailure`.

All contracts use hand-written Protobuf field encoding and the existing
verified atomic writer: create-new temporary write, flush, `sync_all`,
temporary reopen/decode/digest validation, atomic rename, and final
reopen/decode/digest validation. An identical completed replay performs zero
writes, model refits, prediction computations, and metric recomputations.
Runtime artifacts remain ignored and uncommitted.

Every public aggregate carries:

```text
HistoricalResearchOnly
QualifiedSixNotFullEight
NotIndependentLiveEvidence
NotTradingAuthority
```

The replay journal digest is `4d989dea3d3e9572`; the final public report
digest is `d66027c0d2320d13`.

## Verification

Formatting and workspace checks passed in Default and Metal configurations.
Full sequential workspace testing passed with `1,056 + 404 + 12` tests under
Default and `1,057 + 404 + 12` under Metal. The separately run focused
prospective, historical replay, multi-timeframe foundation, macro forensic,
and Qualified-Six suites passed in both configurations with 96, 43, 46, 44,
and 51 tests respectively.
