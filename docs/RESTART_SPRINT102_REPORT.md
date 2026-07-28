# Restart Sprint 102 Report

## Prerequisite and authority

PR #32 was reviewed and merged into authoritative `main` as
`b56e93d5fbfa27dc4860747a988207ac38a0fdc6`. The completed live lane reopened
with two completed and scorable events, pause state
`PausedAfterCompletedEpochTwo`, no epoch three, and protected before-state
digest `d829b4817c6f19d0`.

The frozen source identities reopened as:

| Evidence | Digest |
|---|---|
| Label forensics | `dc1db01318ab180f` |
| Compact-feature forensics | `02bb79cbc18c34c4` |
| Challenger design | `0d1077c9c65fd8cf` |
| Screening registration | `56dbdee4766edaaa` |
| Screening gate | `ccd9763e73e60081` |

T10 remained `StableEnoughForFutureScreening`; T30 and T60 remained
`ExcessiveTemporalInstability`. Authorization `68a275ea51dc7443` is
semantically limited to T10 development and validation. It sets historical
holdout, T30, T60, network, live, governance, and trading execution authority
to false before any model fit.

The T10 chronological boundary contains 18,098 development, 3,878 validation,
and 3,879 sealed-holdout events. The registration-derived shared minimum
training support is 1,024 and the maximum chronological window is 4,096. The
fixed deterministic policy uses seed 7, four epochs, batch size 64, standard
L2 for C2/C4, exactly `4×` L2 for C3, and a chronological 80/20 C4 split.

## Partition execution

| Partition | Boundary | Training-only | Predicted | Scorable | Neutral | Invalid | Daily refits | Insufficient days |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Development | 18,098 | 1,439 | 16,659 | 16,535 | 124 | 0 | 116 | 10 |
| Validation | 3,878 | 0 | 3,878 | 3,848 | 30 | 0 | 28 | 0 |

Development and validation stayed separate. Every eligible day used only
previously revealed development targets. Validation labels fitted no model,
normalizer, or calibrator.

## Participant metrics

| Participant | Partition | Mean Brier | Correctness | Paired mean Δ vs C0 | Paired median Δ vs C0 | Weighted calibration gap | Empty bins | Collapse | Saturation |
|---|---|---:|---:|---:|---:|---:|---:|---|---|
| C0 | Development | 0.250142138 | 0.498760206 | 0 | 0 | 0.008482583 | 8 | BenchmarkExempt | NotSaturated |
| C0 | Validation | 0.249741520 | 0.517931393 | 0 | 0 | 0.007925815 | 9 | BenchmarkExempt | NotSaturated |
| C1 | Development | 0.249676827 | 0.515996371 | -0.000465311 | -0.000715149 | 0.004562823 | 3 | NotCollapsed | NotSaturated |
| C1 | Validation | 0.250144164 | 0.506496881 | 0.000402644 | 0.000170824 | 0.018002303 | 7 | NotCollapsed | NotSaturated |
| C2 | Development | 0.250587410 | 0.521136982 | 0.000445272 | -0.002809410 | 0.020032064 | 1 | NotCollapsed | LowBoundarySaturation |
| C2 | Validation | 0.252722945 | 0.510135135 | 0.002981425 | -0.000642310 | 0.038090982 | 1 | NotCollapsed | NotSaturated |
| C3 | Development | 0.250557323 | 0.520774116 | 0.000415185 | -0.002809808 | 0.020042930 | 1 | NotCollapsed | NotSaturated |
| C3 | Validation | 0.252651354 | 0.509615385 | 0.002909834 | -0.000564924 | 0.038058848 | 1 | NotCollapsed | NotSaturated |
| C4 | Development | 0.250204373 | 0.492410039 | 0.000062235 | 0.000051642 | 0.012262301 | 6 | NotCollapsed | HighBoundarySaturation |
| C4 | Validation | 0.249736476 | 0.517931393 | -0.000005044 | -0.000576889 | 0.004647494 | 8 | NotCollapsed | NotSaturated |

All prediction and metric values were finite. C0 was explicitly exempt from
collapse classification. C2 development hit the registered low-boundary
saturation condition; C4 development hit the registered high-boundary
saturation condition.

## Comparisons and eligibility

| Challenger | Development vs C0 | Validation vs C0 | Overall | Eligibility |
|---|---|---|---|---|
| C1 | Lower Brier | Higher Brier | Mixed | Ineligible |
| C2 | Higher Brier | Higher Brier | Higher Brier | Ineligible |
| C3 | Higher Brier | Higher Brier | Higher Brier | Ineligible |
| C4 | Higher Brier | Lower Brier | Mixed | Ineligible |

Contribution receipts classified C2 versus C1 as higher Brier in both
partitions. C3 versus C2 had lower Brier in both partitions, and C4 versus C2
had lower Brier in both partitions. Contribution improvements cannot override
the primary C0 gate.

C1 failed the validation lower-Brier condition. C2 failed both lower-Brier
conditions and the development saturation condition. C3 failed both
lower-Brier conditions. C4 failed the development lower-Brier and saturation
conditions. All four retained sufficient paired support, finite predictions
and metrics, clean chronology/leakage/integrity, no result-selected mutation,
and zero holdout access.

The deterministic cohort receipt `0a6360694a7d8117` therefore records
`NoEligibleT10HoldoutCohort`, with no participant and no holdout execution
authority.

## Safety, determinism, and conclusions

Both partition aggregates passed chronology, leakage, integrity, and
prediction-before-reveal audits. T10 sealed-holdout prediction, label, and
metric counts are zero. T30 and T60 model, calibration, prediction, evaluation,
and holdout counts are zero. Month/year loads, network and live requests,
live outcomes, live predictions/evaluations, parameter and normalizer changes,
winner selection, ranking, reward, penalty, Chair, vote, paper-trading, and
live-trading counts are zero. Full-Eight remains blocked and the live roster,
counts, parameters, pause, and protected artifacts remain unchanged.

The development and validation aggregate digests are `d0eacba7eea61f23` and
`2278dba4e330e175`. Deterministic replay digest `1e238431ed660a1d` and public
report digest `c3141bf6324ebb59` reopened with zero writes, fits, calibrations,
predictions, target reveals, evaluations, or metric computations.

This run proves only that the preregistered T10 historical development and
validation screening executed deterministically within its closed authority
boundary. It does not prove sealed-holdout performance, independent historical
confirmation, prospective generalization, live superiority, a winner, reward
or Chair effectiveness, official Mamba-3 behavior, or paper/live-trading
readiness. The next single step, if separately authorized, is to revise the
preregistered design; there is no eligible cohort to evaluate on sealed
holdout.

## Verification

Formatting and Default/Metal workspace checks passed. Complete single-threaded
test counts were:

| Configuration | Library | Auxiliary | Integration |
|---|---:|---:|---:|
| Default | 1,271 | 404 | 12 |
| Metal | 1,272 | 404 | 12 |

Focused Default and Metal runs both passed for live prospective (144),
Sprint 100 event-two close (49), historical replay (43), multi-timeframe
foundation (46), macro forensics (44), Qualified-Six replay (51),
Qualified-Six diagnostics (48), Sprint 101 label/challenger design (57), and
Sprint 102 T10 screening (62). Runtime evidence remains private, ignored, and
uncommitted.
