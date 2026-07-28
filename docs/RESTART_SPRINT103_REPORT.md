# Restart Sprint 103 Report

## Baseline

PR #33 was reviewed against its complete patch and critical execution paths,
verified under Default and Metal, marked ready, and merged as
`f3a8ec255add0588c395700f605f6170188cdf49`. The authoritative `main` branch was
then synchronized and reverified.

The frozen Sprint 102 identities remain:

- execution authorization: `68a275ea51dc7443`;
- label forensics: `dc1db01318ab180f`;
- compact feature forensics: `02bb79cbc18c34c4`;
- challenger design: `0d1077c9c65fd8cf`;
- screening registration: `56dbdee4766edaaa`;
- screening gate: `ccd9763e73e60081`;
- completed replay: `1e238431ed660a1d`;
- screening report: `c3141bf6324ebb59`;
- development aggregate: `d0eacba7eea61f23`;
- validation aggregate: `2278dba4e330e175`;
- empty T10 holdout cohort: `0a6360694a7d8117`.

No Sprint 102 challenger passed. No eligible T10 holdout cohort exists. The live
lane remains completed and paused after epoch two, and epoch three remains
absent.

## Sprint 103 operation

Former development and validation are reclassified additively as consumed
research-design evidence. Their original receipts are not rewritten.

The untouched T10 boundary is split chronologically using timestamps and event
identities only. The older half is fresh challenger validation and the newer
half is final sealed holdout. No label, prediction, metric, return, correctness,
or regime value participates in the split. Both children remain closed.

Failure-forensics registration is persisted before consumed private
prediction/evaluation values are reopened. Development-derived magnitude
quintiles are persisted and reopened before validation assignment. Confidence
bands are fixed. Only aggregate diagnostics and canonical C1–C4 dispositions
are public.

The actionability label and selection rule are persisted before candidate labels
are derived. Candidate multipliers are exactly `0.25`, `0.50`, and `1.00`;
selection uses consumed evidence only and chooses the largest multiplier that
passes frozen support, prevalence, and drift rules.

The two-stage selective system is registered only when threshold and future
fresh-support conditions pass. It is not trained or evaluated. Opportunity,
direction, and end-to-end metric contracts are separate, and no P&L metric is
registered.

## Verified Sprint 103 result

The protected before-state digest remained `d829b4817c6f19d0`. The immutable
parent holdout digest is `3b67b9a3e3952aeb`, and its derived event count is
3,879. The additive evidence-use receipt is `e29f103689ce07a1`.

The timestamp-only split produced:

- fresh challenger validation: 1,939 events;
- final sealed holdout: 1,940 events;
- split digest: `a030d99b7e7129fb`;
- label reads: 0;
- prediction reads: 0;
- metric reads: 0;
- fresh-validation execution authorized: false;
- final-holdout execution authorized: false.

The failure-forensics registration is `8b35f40e6efd2557`; its frozen magnitude
boundary digest is `6d343116c9311b66`. The aggregate report is
`43db8e11428ffd7e`, with deterministic replay
`9bce01422387f30f`.

| Participant | Canonical disposition | Tiny-band excess-Brier concentration | Saturation events |
|---|---|---:|---:|
| C1 | `PartitionSpecificSignal` | 0.630885 | 0 |
| C2 | `ProbabilitySaturation` | 0.454672 | 1 |
| C3 | `BroadFeatureUnderperformance` | 0.453147 | 0 |
| C4 | `ProbabilitySaturation` | 0.178072 | 1 |

Each participant has 20,383 paired scorable events across consumed development
and validation. The magnitude-bin aggregate ranges were:

| Participant | Paired Brier delta range | Correctness range | Maximum weighted calibration gap |
|---|---:|---:|---:|
| C1 | -0.001935 to 0.001294 | 0.478315 to 0.538857 | 0.045716 |
| C2 | -0.002264 to 0.005056 | 0.498922 to 0.537950 | 0.019254 |
| C3 | -0.002283 to 0.004946 | 0.500000 to 0.537950 | 0.019402 |
| C4 | -0.000390 to 0.000462 | 0.481403 to 0.537716 | 0.024211 |

The populated confidence-band aggregate ranges were:

| Participant | Paired Brier delta range | Coverage range | Maximum calibration gap |
|---|---:|---:|---:|
| C1 | -0.135026 to 0.133116 | 0.000260 to 0.529886 | 0.610906 |
| C2 | -0.000545 to 0.034353 | 0.005141 to 0.366616 | 0.057381 |
| C3 | -0.000637 to 0.033482 | 0.004959 to 0.369277 | 0.057290 |
| C4 | -0.103067 to 0.771656 | 0.000060 to 0.975312 | 0.999999 |

The actionability-label registration is `c4a97a34207fa0bc`, and the frozen
selection policy is `0db4f32471363228`. Events without a complete registered
145-candle past context were ineligible; no missing context was replaced by the
zero-volatility floor. The aggregate candidate results were:

| k | Partition | Eligible | Up | Down | Abstain | Up / Down / Abstain prevalence |
|---:|---|---:|---:|---:|---:|---|
| 0.25 | Development | 17,954 | 6,508 | 6,578 | 4,868 | 0.362482 / 0.366381 / 0.271137 |
| 0.25 | Validation | 3,878 | 1,395 | 1,510 | 973 | 0.359722 / 0.389376 / 0.250903 |
| 0.50 | Development | 17,954 | 4,603 | 4,627 | 8,724 | 0.256377 / 0.257714 / 0.485908 |
| 0.50 | Validation | 3,878 | 993 | 1,069 | 1,816 | 0.256060 / 0.275658 / 0.468283 |
| 1.00 | Development | 17,954 | 2,070 | 2,238 | 13,646 | 0.115295 / 0.124652 / 0.760053 |
| 1.00 | Validation | 3,878 | 487 | 554 | 2,837 | 0.125580 / 0.142857 / 0.731563 |

All candidates had zero zero-volatility-floor events. The maximum class
prevalence ranges across daily / weekly / monthly / rolling-144 / rolling-1008
groups were:

| k | Partition | Daily | Weekly | Monthly | Rolling 144 | Rolling 1,008 |
|---:|---|---:|---:|---:|---:|---:|
| 0.25 | Development | 0.312500 | 0.074056 | 0.046441 | 0.368056 | 0.087302 |
| 0.25 | Validation | 0.236111 | 0.107336 | 0.101397 | 0.305556 | 0.105159 |
| 0.50 | Development | 0.379711 | 0.074869 | 0.053970 | 0.430556 | 0.096230 |
| 0.50 | Validation | 0.319444 | 0.121929 | 0.076042 | 0.381944 | 0.113095 |
| 1.00 | Development | 0.371261 | 0.077900 | 0.062431 | 0.451389 | 0.098214 |
| 1.00 | Validation | 0.444444 | 0.101887 | 0.048063 | 0.375000 | 0.087302 |

The largest frozen-rule candidate that passed is `k=0.50`; `k=1.00` failed the
registered class-support bounds. The immutable selection receipt is
`291d95c4875ae650`. Fresh-validation support is 1,939 against the frozen minimum
of 1,024, recorded as `SufficientSupport`, without opening that evidence.

Registered participant identities and digests are:

- O0 `0ca1d52c125fe1ab`, O1 `e15885d0ddaf82a5`, O2 `37f721df5ace4df4`;
- D0 `b7ae36201765b682`, D1 `54cbcb50888463a3`, D2 `78d73818a8c141b0`;
- S0 `ceb67721cb1d9701`, S1 `9a2366b3c6fb25a5`, S2 `fee4cc5ad798ed1e`.

The combined two-stage registration is `d353490a7b8d8ff3`. The future training
policy is `f5451c24c42f32b3`, the unexecuted fresh-validation gate is
`6a9f79146d491734`, and the separately unauthorized final-holdout gate is
`6bd087ec28f4d934`. The final design report is `191093c82950ff7e`, with
deterministic replay `49a2d518265d177a`.

Both completed replays reproduced their report and replay digests with zero new
writes, error computations, label computations, model fits, predictions,
evaluations, metrics, sealed-evidence reads, or authority actions.

## Preserved boundaries

- fresh challenger validation remains unread and unexecuted;
- final holdout remains unread, unexecuted, and unauthorized;
- T30 and T60 remain blocked with zero execution;
- month and year views remain inaccessible;
- Full-Eight A3 remains blocked;
- live participants, parameters, normalizers, counts, and pause remain
  unchanged;
- reward, penalty, Chair, vote, network, and trading counters remain zero.

## Evidence limits

This Sprint proves deterministic registration, consumed-evidence diagnostics,
timestamp-only splitting, aggregate actionability-label stability, zero new
model execution, and preservation of protected state.

It does not prove selective-model performance, fresh-validation performance,
final-holdout performance, prospective generalization, live superiority,
reward effectiveness, governance learning, paper-trading readiness, or
live-trading readiness.

## Changed source and verification

The implementation extends the existing CLI, model exports, qualified T10
evidence reader, past-only volatility reuse, and consumed Sprint 102 evidence
reader. It adds focused failure-forensics and actionability-design modules plus
the three Sprint 103 documents. Runtime evidence remains ignored and
uncommitted.

Final local verification passed:

- `cargo fmt --all --check`;
- Default workspace check;
- Default Sprint 103 focused tests: 71 passed;
- full Default tests: 1,342 + 404 + 12 passed;
- Metal workspace check;
- Metal Sprint 103 focused tests: 71 passed;
- full Metal tests: 1,343 + 404 + 12 passed;
- source and staged whitespace checks.

The only compiler output was four pre-existing dead-code warnings outside the
Sprint 103 implementation.
