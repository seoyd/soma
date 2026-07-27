# Momentum Qualified-Six Diagnostics V1

## Scope and evidence class

This lane is `PostResultDiagnosticOnly`. It reopens the completed
Qualified-Six historical replay and measures stability, calibration,
distribution, prevalence, past-only regimes, and daily refit drift without
training or executing a new model.

The source replay remains identified by:

```text
registration: e54c65c8ecbfef85
journal:      4d989dea3d3e9572
report:       d66027c0d2320d13
```

Only development and validation source evidence is readable. The sealed
holdout, live outcomes, month and year views, model coefficients, normalizer
values, and event-level public output remain inaccessible. Every public
diagnostic result carries:

```text
HistoricalResearchOnly
PostResultDiagnosticOnly
QualifiedSixNotFullEight
NotIndependentEvidence
NotHoldoutEvidence
NotTradingAuthority
```

## Frozen diagnostic contract

Registration `6a107c3f185c1a47` was atomically persisted and reopened before
private event evidence was loaded. It binds the completed source replay,
Q0-Q4 identities, development and validation partitions, paired epsilon,
UTC calendar grouping, fixed rolling windows, ten fixed calibration bins,
existing collapse and clamp thresholds, scorable-label prevalence,
development-only volatility terciles, a past-only 16-candle daily trend,
minimum support 512, deterministic drift thresholds, and the conservative
holdout gate.

The volatility receipt was derived only from development, persisted, and
reopened before validation regime assignment:

```text
low upper:    0.0013737119215777125
medium upper: 0.002005377018559812
validation threshold inputs: 0
```

Micro volatility is the population standard deviation of the preceding 144
simple returns from 145 already-closed 10-minute candles. Daily trend is the
cumulative close return over the preceding 16 already-closed daily candles.
Neither regime definition reads the current target.

## Paired event-level Brier aggregates

Delta is participant Brier minus Q0 Brier, so a negative value is lower
Brier than Q0. Neutral targets are excluded and development and validation
remain separate.

| Participant | Partition | Paired | Mean delta | Median delta | Positive | Negative |
|---|---|---:|---:|---:|---:|---:|
| Q1 | Development | 17,395 | -0.000274023 | -0.000787897 | 8,423 | 8,972 |
| Q1 | Validation | 3,846 | +0.000033191 | -0.001465382 | 1,847 | 1,999 |
| Q2 | Development | 17,395 | +0.000133546 | -0.001035946 | 8,438 | 8,957 |
| Q2 | Validation | 3,846 | -0.000276449 | -0.003079346 | 1,814 | 2,032 |
| Q3 | Development | 17,395 | +0.001186312 | +0.000005822 | 8,709 | 8,686 |
| Q3 | Validation | 3,846 | +0.000393580 | -0.000175492 | 1,893 | 1,953 |
| Q4 | Development | 17,395 | +0.001032309 | -0.001116016 | 8,448 | 8,947 |
| Q4 | Validation | 3,846 | +0.000399756 | -0.002201539 | 1,838 | 2,008 |

Q1 is lower on development only, Q2 is lower on validation only, and Q3
and Q4 are higher across both partition means. Sign counts and medians show
why a single aggregate is insufficient evidence of temporal stability.

## Calendar stability

Each cell is `lower groups / higher groups` versus Q0; equivalent-group
counts were zero.

| Participant | Development day | week | month | Validation day | week | month |
|---|---:|---:|---:|---:|---:|---:|
| Q1 | 77/45 | 10/8 | 2/3 | 15/13 | 3/2 | 1/1 |
| Q2 | 73/49 | 9/9 | 2/3 | 14/14 | 2/3 | 1/1 |
| Q3 | 54/68 | 4/14 | 0/5 | 15/13 | 2/3 | 1/1 |
| Q4 | 62/60 | 9/9 | 2/3 | 14/14 | 2/3 | 1/1 |

Development contains 122 UTC days, 18 UTC weeks, and five UTC months.
Validation contains 28 UTC days, five UTC weeks, and two UTC months.

## Rolling stability

The table reports non-overlapping windows as
`eligible: lower/higher, median delta`. Actual timestamp spans are persisted
because neutral events can expand the elapsed span represented by a fixed
number of scorable events.

| Participant | Partition | 144-event | 1,008-event |
|---|---|---|---|
| Q1 | Development | 120: 74/46, -0.000394855 | 17: 11/6, -0.000546938 |
| Q1 | Validation | 26: 12/14, +0.000248013 | 3: 2/1, -0.000148816 |
| Q2 | Development | 120: 70/50, -0.000915953 | 17: 11/6, -0.000522940 |
| Q2 | Validation | 26: 15/11, -0.000208691 | 3: 2/1, -0.000346999 |
| Q3 | Development | 120: 48/72, +0.000199531 | 17: 3/14, +0.000513859 |
| Q3 | Validation | 26: 11/15, +0.000032592 | 3: 1/2, +0.000983250 |
| Q4 | Development | 120: 62/58, -0.000174898 | 17: 8/9, +0.000240423 |
| Q4 | Validation | 26: 10/16, +0.000386097 | 3: 1/2, +0.000439279 |

Step-one rolling diagnostics are also persisted for every participant,
partition, and window size. They contain 17,252/16,388 development windows
and 3,703/2,839 validation windows for 144/1,008 events respectively. Their
sign trajectories remain mixed; notably Q3 and Q4 have higher-Brier
majorities in both validation window sizes.

## Calibration and probability distributions

Weighted ten-bin calibration gaps:

| Participant | Development | Validation |
|---|---:|---:|
| Q0 | 0.007478073 | 0.014637140 |
| Q1 | 0.004524348 | 0.002997779 |
| Q2 | 0.010634264 | 0.008624272 |
| Q3 | 0.022889002 | 0.011541675 |
| Q4 | 0.021931522 | 0.021871892 |

Probability summaries are `minimum / median / maximum / mean / standard
deviation`:

| Participant | Development | Validation |
|---|---|---|
| Q0 | 0.463508 / 0.498984 / 0.501967 / 0.494418 / 0.009816 | 0.495595 / 0.496295 / 0.498484 / 0.496439 / 0.000748 |
| Q1 | 0.179866 / 0.496137 / 0.819083 / 0.499499 / 0.029743 | 0.398936 / 0.481496 / 0.688691 / 0.483943 / 0.020923 |
| Q2 | 0.085088 / 0.497381 / 0.859170 / 0.500037 / 0.038552 | 0.346517 / 0.479289 / 0.708301 / 0.483806 / 0.035904 |
| Q3 | 0.224220 / 0.499871 / 0.620690 / 0.497832 / 0.037975 | 0.410285 / 0.485034 / 0.510783 / 0.477360 / 0.027406 |
| Q4 | 0.087187 / 0.499121 / 0.838922 / 0.498898 / 0.054267 | 0.328360 / 0.478732 / 0.768325 / 0.478838 / 0.048137 |

Q0 is explicitly benchmark-exempt from the learned-candidate collapse
gate. Q1-Q4 are `NotCollapsed` and `NotSaturated`; all nonfinite and extreme
boundary counts are zero.

## Scorable-label prevalence

| Partition | Scorable | Positive | Negative | Prevalence |
|---|---:|---:|---:|---:|
| Development | 17,395 | 8,684 | 8,711 | 0.499223915 |
| Validation | 3,846 | 1,853 | 1,993 | 0.481799272 |

Development UTC-day prevalence ranges from 0.388889 to 0.608392, week from
0.457746 to 0.526000, and month from 0.463028 to 0.504164. Validation
UTC-day prevalence ranges from 0.333333 to 0.535211, week from 0.459605 to
0.502498, and month from 0.481596 to 0.484321. Only scorable labels
contribute.

## Past-only regime diagnostics

Across learned participants, the relation counts are:

| Partition | Dimension | Lower | Higher | Insufficient |
|---|---|---:|---:|---:|
| Development | Micro volatility | 4 | 8 | 0 |
| Development | Daily trend | 3 | 5 | 4 |
| Development | Volatility × trend | 8 | 16 | 12 |
| Validation | Micro volatility | 4 | 8 | 0 |
| Validation | Daily trend | 4 | 4 | 4 |
| Validation | Volatility × trend | 5 | 7 | 24 |

The flat daily-trend cells have no support in this replay and are explicitly
classified `InsufficientDiagnosticSupport`. The minimum-support policy
therefore does not silently turn sparse regime cells into evidence.

## Daily refit and normalizer drift

All parameter, normalizer, and finite-training-integrity receipts reopened
successfully. Coefficients and normalizer values remain private.

| Participant | Partition | Refits | Max parameter-norm change | Max normalizer shift | Status |
|---|---|---:|---:|---:|---|
| Q1 | Development | 122 | 0.490716 | 0.003397 | Moderate |
| Q2 | Development | 122 | 0.370380 | 0.005894 | Moderate |
| Q3 | Development | 122 | 0.851351 | 0.136154 | High |
| Q4 | Development | 122 | 0.291220 | 0.078621 | Moderate |
| Q1 | Validation | 28 | 0.441019 | 0.002064 | Moderate |
| Q2 | Validation | 28 | 0.128726 | 0.001968 | Moderate |
| Q3 | Validation | 28 | 0.346943 | 0.048241 | Moderate |
| Q4 | Validation | 28 | 0.272729 | 0.027865 | Moderate |

No partition-boundary shift, probability collapse, nonfinite parameter
summary, or nonfinite normalizer summary was observed.

## Partition stability and sealed-holdout gate

| Participant | Stability classification | Holdout eligibility |
|---|---|---|
| Q1 | `DevelopmentOnlyLowerBrier` | `NotEligibleForSealedHoldout` |
| Q2 | `ValidationOnlyLowerBrier` | `NotEligibleForSealedHoldout` |
| Q3 | `HigherBrierAcrossDevelopmentAndValidation` | `NotEligibleForSealedHoldout` |
| Q4 | `HigherBrierAcrossDevelopmentAndValidation` | `NotEligibleForSealedHoldout` |

The sealed holdout remains closed: label reads, prediction reads, metric
reads, and holdout execution modes are all zero. Diagnostics do not make a
candidate independently qualified.

## Challenger requirements

Receipt `27322215c5d802cd` keeps Q0 mandatory, forbids Full-Eight and
month/year claims, forbids complexity escalation, interaction expansion, and
sequence-model escalation, and requires label forensics, calibration repair,
regime stability, and lower Brier in both development and validation.

The micro block is the `PrimaryDiagnosticTarget`; the qualified macro
addition is `DeprioritizedByCurrentEvidence`. This is a requirements receipt,
not a model selection. It authorizes neither new-model execution nor
holdout execution.

## Persistence, determinism, and CLI

```text
--momentum-mtf-qualified-six-diagnostics --status --output-format text|json
--momentum-mtf-qualified-six-diagnostics --dry-run --output-format text|json
--momentum-mtf-qualified-six-diagnostics --register --execute-local --output-format json
--momentum-mtf-qualified-six-challenger-requirements --status --output-format text|json
```

There is no holdout, live, network, authority, or challenger-execution mode.
All policies, registration, threshold receipt, individual aggregates, suite
receipts, journal, challenger requirements, and final report use hand-written
Protobuf with the existing verified atomic writer.

The first completed execution wrote 280 artifacts and performed 21,810
diagnostic computations while performing zero model refits, predictions, or
evaluations. An identical completed execution reopened the report with zero
writes and zero diagnostic, refit, prediction, or evaluation computations;
all 280 artifacts were accounted for as duplicates.

```text
diagnostic journal: c59d839eaa5dba34
diagnostic report:  748dc5291fb165ab
```

No winner, ranking, reward, penalty, Chair action, trading action, network
attempt, live-outcome opening, or protected-artifact mutation occurred.

Live event two later completed in the separate prospective lane without
reading the sealed historical holdout or changing this diagnostic store,
replay identity, candidate classifications, or challenger requirements. Its
result does not alter the historical conclusions above.

## Verification

All 48 focused diagnostic tests passed under Default and Metal. Formatting
and workspace checks passed in both configurations. Full sequential
workspace testing passed with `1,104 + 404 + 12` tests under Default and
`1,105 + 404 + 12` under Metal, together with the separately executed
prospective, historical replay, multi-timeframe foundation, macro forensic,
and Qualified-Six replay suites.
