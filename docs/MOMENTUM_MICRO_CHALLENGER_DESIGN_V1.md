# Momentum Micro Challenger Design V1

## Scope

This component completes feature diagnostics and preregisters a bounded
future challenger family. It performs no logistic fit, calibration fit,
prediction, Brier evaluation, partition aggregate, winner selection, holdout
execution, or live action.

The evidence class and public labels are identical to Momentum Micro Label
Forensics V1. All output is post-result design evidence, not confirmation.

## Existing feature-policy audits

The existing Q1 ten-minute anchor and Q2 four-view micro policies are reopened
from the Qualified-Six source. Their feature order, source timeframes, finite
status, constant or near-constant count, duplicate semantic count, normalizer
identity, normalizer finite status, development/validation availability, and
closed-candle completeness are audited without publishing feature vectors or
normalizer values.

Redundancy uses a fixed absolute Pearson threshold of `0.98`, derived on
development and frozen before validation confirmation. Development-derived
equal-frequency decile boundaries are persisted and reopened before
validation assignment. Per-feature receipts contain aggregate bin support,
PSI-style drift, mean-shift class, standard-deviation-shift class,
out-of-development-range count, and integrity status.

All Q1 and Q2 daily normalizer receipts are reopened. The public audit contains
only aggregate refit count, finite status, maximum/median/95th-percentile
shift, sign-change count, partition-boundary shift status, and a digest
trajectory. Prior drift evidence is immutable.

## Compact Micro Feature Policy V1

The fixed schema contains 69 ordered features:

```text
4 timeframes × 16 per-timeframe features + 5 cross-timeframe features
```

Included timeframes are exactly `1m`, `3m`, `5m`, and `10m`. Daily, weekly,
monthly, and yearly inputs are excluded.

Each timeframe contributes:

- log returns over 1, 3, 6, and 12 closed candles;
- realized standard deviation over 6, 12, and 16 past returns;
- log high/low range, body/range, upper wick/range, lower wick/range, and
  close location;
- one-candle log volume change, 16-candle volume z-score, and 16-candle
  trade-value z-score;
- a deterministic least-squares slope over 16 normalized closes.

Zero candle ranges and zero cross-timeframe denominators use registered,
deterministic finite fallbacks.

The five cross-timeframe features are exactly:

```text
return-sign agreement count
latest-return dispersion
1m/10m realized-volatility ratio
3m/10m realized-volatility ratio
1m/10m normalized-volume ratio
```

Target-, validation-, and holdout-selected feature flags are all false. The
integrity replay runs the extractor across development and validation only and
requires zero future, partial-candle, and holdout access.

## Future tasks and participants

Exactly two future tasks are registered:

- T10: completed 10-minute cadence and one completed 10-minute target candle.
- T30: non-overlapping 30-minute cadence and three completed 10-minute target
  candles.

Each task uses an oldest-70%, next-15%, newest-15% chronological partition.
The newest partition is a sealed holdout. Boundaries use timestamps and event
counts only and read no labels.

Five participants are preregistered independently for each task:

| ID | Fixed design |
|---|---|
| C0 | Task-specific training-prevalence constant |
| C1 | Ten-minute anchor with fresh task-specific parameters |
| C2 | Compact Micro bounded logistic |
| C3 | Identical compact schema with fixed `4×` standard L2 |
| C4 | Identical compact schema with nested training-only 80/20 calibration |

Validation and holdout cannot fit calibration. T60 remains diagnostic-only.

The future screening gate requires lower mean Brier than C0 in both
development and validation, finite predictions and metrics, sufficient paired
support, and clean chronology, leakage, probability, and integrity audits.
Correctness cannot override a Brier failure.

The subsequent T10-only execution is recorded in
`MOMENTUM_T10_MICRO_SCREENING_V1.md`. It did not execute T30 or T60, did not
open holdout, and produced no eligible T10 holdout cohort. The frozen
registration and gate in this document remain unchanged.

## Persistence, determinism, and CLI

Feature registration, source audits, persisted decile policies, redundancy and
shift audits, compact policy and integrity receipts, task boundaries, task and
participant registrations, screening gate, journal, and public reports use
hand-written Protobuf and verified atomic persistence.

```text
--momentum-micro-feature-forensics --status --output-format text|json
--momentum-micro-feature-forensics --execute-local --output-format json
--momentum-micro-challenger-registration --status --output-format text|json
--momentum-micro-challenger-registration --register --execute-local --output-format json
```

Completed replay returns the same semantic report identity with zero writes,
feature computations, registrations, model fits, predictions, or evaluations.
