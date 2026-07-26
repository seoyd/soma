# Sprint 96 Implementation Report

## PR 26 review, correction, and merge

The complete PR 26 diff and all comments were reviewed. The completed-chain
validator already avoided filesystem order as authority, but the validator
itself did not prove that all capsule seal bindings resolved exactly once
against the frozen three-participant roster.

The PR was corrected before merge to require exact seal/prediction bindings,
reject duplicate, missing, mismatched, and extra seals, preserve reversed-file
order, and bind the exact frozen roster. Focused and full Default and Metal
verification passed. Commit `375c334` was added without rewriting the two
required commits. PR 26 was marked ready and merged as
`3e587b34e7f3a3d10c36159d6ace051a02c01a19`; its remote working branch was
deleted. Local `main` and `origin/main` were synchronized and fully verified.

## Live pause and protected state

The authoritative live chain reopened at epoch two with
`PredictionAlreadySealed`, one completed and one scorable event, three
prediction seals, one input attempt, zero retries, and
`IneligibleMinimumSamples`.

The live continuation pause is `1f6b1750646e9b59`. No epoch three was
registered. The protected live tree contained 176 artifacts before the
historical workflow and remained byte-identical after every mode. The active
three-member roster also remained unchanged.

Event-two outcome requests, openings, label reads, metrics, and evaluations
remain zero. Participant changes, parameter updates, normalizer refits,
feature-policy changes, winner selections, rankings, rewards, penalties,
Chair decisions, votes, voice/tier/cooldown/promotion/quarantine changes, and
paper/live executions are all zero.

## Foundation and acquisition result

The fixed research set is `1m, 3m, 5m, 10m, 1d, 1w, 1mo, 1y`.
Only `1m` and `1d` are canonical. Foundation registration is
`a7655ab907fb428d`; acquisition plan is `e3ec06d69f0b3d56`.

The pre-transport budget was exactly 1,400. Actual acquisition consumed 1,314
public requests with concurrency one and zero retries:

- 1,293 canonical minute pages;
- 15 older daily pages;
- six native cross-check pages.

There are 1,314 verified page receipts and 1,314 checkpoints. Failed pages are
zero. Repeated execution skips verified pages.

Canonical `1m` contains 258,557 observed rows over 180 complete UTC days,
643 verified no-trade intervals, zero missing evidence, 64 chunks, dataset
`224c70d04c4eb0ad`, and index `3695f8b89ef277a7`.

Canonical `1d` contains 3,216 observed rows from the provider's actual
earliest returned day through the frozen snapshot boundary, zero no-trade or
missing intervals, one chunk, dataset `2b0bc50177b20d6e`, and index
`0d8e28078ba9df30`.

## Derived and native comparison result

The deterministic derived views are:

- `3m`: 86,194, index `67e6fd8c5ac5465f`;
- `5m`: 51,717, index `83a264399b680fa3`;
- `10m`: 25,860, index `18de0f3b7e090c82`;
- `1w`: 459, index `850ceefd3ab167d3`;
- `1mo`: 105, index `4097b2993e81d0f7`;
- `1y`: 8, index `4fe08a915d802456`.

All derived missing-evidence counters are zero. Intraday derived views bind
25, 28, and 43 no-trade base intervals respectively; macro views bind zero.

Native `3m`, `5m`, `10m`, and `1w` samples contain only exact or registered
tolerance matches. Native `1mo` has 15 integrity failures and native `1y` has
four. The frozen OHLC exact rule was not weakened. These mismatches block the
future model replay and remain an explicit unresolved research risk.

## Causal replay and future registration

Protocol replay `385a3876663ecabf` sealed 25,856 events. All selected candles
were closed, future and partial-candle access remained zero, prediction
identity was sealed before target-timestamp reveal, and no target value,
metric, score, training, or performance claim was produced.

Coverage after the daily base boundary was classified as missing evidence:
864 daily views and 432 weekly views. No forward-fill or synthetic OHLCV was
used.

Future three-task registration is `99376903a61d45a7`; A0–A5 ablation
registration is `01d54665e30c494a`; sealed holdout is
`112a88137cf7db2f`. The 70/15/15 partition has 18,099 development, 3,878
validation, and 3,879 holdout events. Holdout labels and metrics remain
closed. Duplicate protocol replay performs zero writes.

## Verification boundary

The implementation adds 45 focused contracts covering the required
timeframe, aggregation, acquisition, persistence, causality, holdout,
authority, malformed-Protobuf, deterministic replay, and output invariants.
All Soma Rust commands run sequentially with one build job and one test
thread.

Focused verification passed in both configurations: live 54/54, existing
historical replay 43/43, and multi-timeframe history 45/45. Full Default
passed 961 + 404 + 12 tests; full Metal passed 962 + 404 + 12 tests.
Default and Metal workspace checks, formatting, and diff checks also passed.

This Sprint proves the data and causality foundation only. It does not prove
model improvement, independent holdout performance, future generalization,
participant superiority, reward effectiveness, Chair learning, or trading
readiness.

The next single step is to resolve or formally disposition the native
month/year mismatch under a new registration before executing any
multi-timeframe model replay.
