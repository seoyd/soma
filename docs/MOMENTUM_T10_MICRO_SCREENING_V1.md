# Momentum T10 Compact Micro Screening V1

This lane performs deterministic, local historical screening for the
preregistered T10 task only. It reopens the frozen label, compact-feature,
challenger-design, registration, and screening-gate evidence before doing any
fit. T30 and T60 have no execution authority.

## Authority boundary

The additive execution authorization permits only development and validation
for the five registered T10 participants. Historical holdout, network, live,
governance, reward, penalty, Chair, paper-trading, and live-trading authority
remain false. The completed live lane remains paused after epoch two, with no
epoch three and no roster or parameter mutation.

The authorization binds these frozen identities:

| Evidence | Digest |
|---|---|
| Label forensics | `dc1db01318ab180f` |
| Compact-feature forensics | `02bb79cbc18c34c4` |
| Challenger design | `0d1077c9c65fd8cf` |
| Screening registration | `56dbdee4766edaaa` |
| Screening gate | `ccd9763e73e60081` |

## Participants and fitting

All participants use one paired chronological event set:

| Participant | Frozen role |
|---|---|
| C0 | Training-window T10 prevalence constant |
| C1 | Fresh T10 anchor logistic |
| C2 | 69-dimensional Compact Micro logistic with standard L2 |
| C3 | The C2 design with exactly `4×` standard L2 |
| C4 | Compact Micro base fit on the oldest 80% and a one-dimensional calibrator fit on the newest 20% |

The 69-dimensional Compact Micro schema is exactly four 16-feature blocks for
1m, 3m, 5m, and 10m, followed by five registered cross-timeframe features.
Month and year views are inaccessible.

The shared minimum support is derived as the next power of two above ten times
the maximum participant dimension, producing 1,024 examples. A C4-eligible
daily window also requires at least 1,024 examples in its oldest-80% base
split and the independently dimension-derived support for its calibrator.
Until both conditions hold, the entire UTC day is excluded for all five
participants.

At every eligible UTC boundary the implementation uses only development
events whose T10 targets are already revealed, preserves chronological order,
keeps at most 4,096 examples, fits training-only normalizers, refits C0-C4,
persists every receipt, reopens the daily bundle, reconstructs all five
participants, and freezes them for that day. There is no within-day refit.
Validation labels never fit a model, normalizer, or calibrator.

## Prediction, evaluation, and gate

Each partition/day has one private prediction shard with five ordered
predictions per event. The complete shard is atomically persisted, reopened,
decoded, and validated before any target is revealed. Neutral and invalid
events remain counted but are excluded from Brier calculations.

Development and validation metrics remain separate. Each participant reports
paired mean and median Brier deltas against C0, correctness, fixed ten-bin
calibration, collapse and saturation diagnostics, and chronology, leakage,
and integrity audits. Contribution receipts compare C2/C1, C3/C2, and C4/C2.

A learned challenger is eligible for a future sealed-holdout evaluation only
if it has lower mean Brier than C0 in both partitions and every frozen
support, finite-value, collapse, saturation, chronology, leakage, integrity,
mutation, and zero-holdout-access condition passes. The deterministic proposed
cohort contains every eligible challenger or is empty. It never grants
holdout execution.

## Persistence and CLI

Authorization, training plans, normalizers, model and calibration receipts,
daily bundles, event plans, prediction and evaluation shards, metrics,
comparisons, eligibility, cohort, journal, and public report use hand-written
Protobuf contracts and the existing verified atomic persistence path.
Completed replay reopens the final report and performs zero new writes, fits,
predictions, target reveals, evaluations, or metric computations.

```text
--momentum-t10-micro-screening --status --output-format text|json
--momentum-t10-micro-screening --dry-run --output-format text|json
--momentum-t10-micro-screening --authorize --execute-local --output-format json
--momentum-t10-micro-screening --execute-local --partition development --output-format json
--momentum-t10-micro-screening --execute-local --partition validation --output-format json
--momentum-t10-holdout-cohort --status --output-format text|json
```

Execution modes require explicit local authority and JSON output. No CLI mode
exists for T10 holdout, T30, T60, network, or live execution.
