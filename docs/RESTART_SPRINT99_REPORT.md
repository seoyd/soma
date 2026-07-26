# Sprint 99 Qualified-Six Stability Forensics Report

## Merge and authoritative before-state

PR #29 was reviewed against its registration, chronology, prediction
ordering, sealed-holdout, authority, persistence, and replay invariants. Its
Default and Metal suites passed before merge. It was then merged as
`79feb47cb49822c416114626abc1d0bd55f5961a`, `main` was synchronized to that
merge, and the same checks passed again. The original PR head
`211142db33920d333fcf4d561ab1163de37eae01` remains in merge history.

The authoritative source replay identities are registration
`e54c65c8ecbfef85`, journal `4d989dea3d3e9572`, and report
`d66027c0d2320d13`. Reopening them reproduced the existing development and
validation aggregates: 17,395 and 3,846 paired scorable events, with the
published Q0-Q4 Brier and correctness results unchanged. The protected live
artifact tree, active roster, sealed second live event, blocked Full-Eight
registration, and unregistered live epoch three formed the protected
before-state.

Before any event-level private value was read, the complete Sprint 99 lane
was declared `PostResultDiagnosticOnly`, its policies and registration were
atomically persisted and reopened, and its source replay identities were
validated.

## Diagnostic registration and paired evidence

Diagnostic registration `6a107c3f185c1a47` binds the completed replay,
participants Q0-Q4, development and validation only, and every fixed
diagnostic policy. It provides historical post-result diagnostics, not
independent or holdout evidence.

| Participant | Development paired/mean/median | Validation paired/mean/median |
|---|---|---|
| Q1 | 17,395 / -0.000274023 / -0.000787897 | 3,846 / +0.000033191 / -0.001465382 |
| Q2 | 17,395 / +0.000133546 / -0.001035946 | 3,846 / -0.000276449 / -0.003079346 |
| Q3 | 17,395 / +0.001186312 / +0.000005822 | 3,846 / +0.000393580 / -0.000175492 |
| Q4 | 17,395 / +0.001032309 / -0.001116016 | 3,846 / +0.000399756 / -0.002201539 |

Positive/negative delta counts are respectively Q1 8,423/8,972 and
1,847/1,999; Q2 8,438/8,957 and 1,814/2,032; Q3 8,709/8,686 and
1,893/1,953; Q4 8,448/8,947 and 1,838/2,008 for development and validation.
Neutral targets are excluded.

## Stability, calibration, and distribution result

UTC day/week/month lower/higher group counts:

| Participant | Development day/week/month | Validation day/week/month |
|---|---|---|
| Q1 | 77/45, 10/8, 2/3 | 15/13, 3/2, 1/1 |
| Q2 | 73/49, 9/9, 2/3 | 14/14, 2/3, 1/1 |
| Q3 | 54/68, 4/14, 0/5 | 15/13, 2/3, 1/1 |
| Q4 | 62/60, 9/9, 2/3 | 14/14, 2/3, 1/1 |

Non-overlapping 144-event lower/higher windows are Q1 74/46 and 12/14,
Q2 70/50 and 15/11, Q3 48/72 and 11/15, and Q4 62/58 and 10/16 for
development and validation. Non-overlapping 1,008-event results are Q1
11/6 and 2/1, Q2 11/6 and 2/1, Q3 3/14 and 1/2, and Q4 8/9 and 1/2.
Step-one 144- and 1,008-event trajectories were also persisted with actual
timestamp spans.

Weighted ten-bin calibration gaps in development/validation are Q0
0.007478073/0.014637140, Q1 0.004524348/0.002997779, Q2
0.010634264/0.008624272, Q3 0.022889002/0.011541675, and Q4
0.021931522/0.021871892.

Q0 is explicitly benchmark-exempt. Q1-Q4 are neither collapsed nor
saturated, with zero nonfinite and extreme-boundary predictions. Their
development/validation probability standard deviations are Q1
0.029743/0.020923, Q2 0.038552/0.035904, Q3 0.037975/0.027406, and Q4
0.054267/0.048137. Full percentile, minimum, median, maximum, and mean
results are recorded in the diagnostic report and public diagnostics
document.

## Prevalence and past-only regimes

Development has 17,395 scorable labels, 8,684 positives, and prevalence
0.499223915. Validation has 3,846 scorable labels, 1,853 positives, and
prevalence 0.481799272. Deterministic UTC-day, week, and month trajectories
were derived only from scorable labels.

Development-only volatility terciles were frozen at low upper
0.0013737119215777125 and medium upper 0.002005377018559812 before any
validation value was used. The volatility, preceding-16-daily-candle trend,
and combined regime diagnostics use only already-closed past candles.

Across learned participants, micro-volatility cells classify 4 lower and 8
higher in each partition. Daily-trend cells classify 3 lower, 5 higher, and
4 insufficient in development, and 4 lower, 4 higher, and 4 insufficient in
validation. Combined cells classify 8 lower, 16 higher, and 12 insufficient
in development, and 5 lower, 7 higher, and 24 insufficient in validation.
All flat-trend sparse cells remain explicitly insufficient rather than being
treated as evidence.

## Daily model drift and partition decisions

All 122 development and 28 validation daily refit receipts reopened exactly.
Parameters, normalizers, and finite-training-integrity summaries are finite
and private values were not published. Maximum parameter-norm changes for
Q1-Q4 are 0.490716, 0.370380, 0.851351, and 0.291220 in development and
0.441019, 0.128726, 0.346943, and 0.272729 in validation. Q3 development is
classified high deterministic drift; the remaining partition/participant
combinations are moderate. There is no partition-boundary shift.

| Participant | Partition stability | Sealed-holdout eligibility |
|---|---|---|
| Q1 | `DevelopmentOnlyLowerBrier` | `NotEligibleForSealedHoldout` |
| Q2 | `ValidationOnlyLowerBrier` | `NotEligibleForSealedHoldout` |
| Q3 | `HigherBrierAcrossDevelopmentAndValidation` | `NotEligibleForSealedHoldout` |
| Q4 | `HigherBrierAcrossDevelopmentAndValidation` | `NotEligibleForSealedHoldout` |

No candidate passed the required first gate of lower Brier in both
development and validation. Diagnostics neither select a winner nor make a
candidate independently qualified.

## Holdout, challenger, live, and authority audits

Sealed-holdout label reads, participant prediction reads, metric reads, and
execution modes are all zero; no holdout aggregate exists. Challenger
requirements digest `27322215c5d802cd` makes Q0 mandatory, sets the micro
block as the primary diagnostic target, deprioritizes the qualified macro
addition, requires label, calibration, regime, and two-partition evidence,
and authorizes no new model or holdout execution.

Full-Eight remains blocked and month/year use remains zero. Live event-two
outcome requests and openings, participant changes, trading actions, winner
and ranking actions, rewards, penalties, Chair decisions, network attempts,
transport constructions, and credential reads remain zero. The active
roster, live counts, and protected artifacts remain unchanged.

The deterministic journal is `c59d839eaa5dba34`; the final report is
`748dc5291fb165ab`. Initial local execution wrote 280 verified artifacts and
performed 21,810 diagnostic computations with zero refit, prediction, and
evaluation computations. Duplicate execution produced the same report with
zero writes and zero computations and accounted for 280 duplicate
artifacts.

## Implementation and verification

Implementation reused the existing replay decoders, partition identities,
manual Protobuf helpers, semantic digests, and verified atomic writer. It
added a focused diagnostic module and small replay/CLI/module bridges; it did
not duplicate the trainer or prediction engine. Runtime diagnostic artifacts
remain ignored and uncommitted.

Changed files:

```text
src/cli.rs
src/model/mod.rs
src/model/momentum_qualified_six_replay_v1.rs
src/model/momentum_qualified_six_diagnostics_v1.rs
docs/MOMENTUM_QUALIFIED_SIX_REPLAY_V1.md
docs/MOMENTUM_QUALIFIED_SIX_DIAGNOSTICS_V1.md
docs/RESTART_SPRINT99_REPORT.md
```

The 48 dedicated diagnostic tests cover registration, source access,
pairing, calendar and rolling stability, calibration and distributions,
prevalence, past-only regimes, private refit evidence, stability and holdout
classification, authority zeros, persistence conflicts, malformed
Protobuf, duplicate execution, and text/JSON agreement.

Formatting and workspace checks passed under Default and Metal. Separately
run focused suites passed under both configurations: prospective 96,
historical replay 43, multi-timeframe foundation 45 plus its prospective-chain
boundary check, macro forensic 44, Qualified-Six replay 51, and
Qualified-Six diagnostics 48. Full sequential workspace testing passed with
`1,104 + 404 + 12` tests under Default and `1,105 + 404 + 12` under Metal.
Both repository boundary audits passed.

## What was and was not proven

This proves deterministic reconstruction of the completed Qualified-Six
development/validation replay, aggregate post-result diagnostics, explicit
mixed temporal behavior, conservative holdout ineligibility, zero protected
access, and deterministic persisted replay of the diagnostic result.

It does not prove sealed-holdout performance, independent validation, future
generalization, live superiority, a winner, reward effectiveness, Chair
learning, official Mamba-3 behavior, or paper/live trading readiness.

The next single step is a separately preregistered challenger design focused
on micro-block label, calibration, and regime stability requirements, with
no model execution until that registration is reviewed.
