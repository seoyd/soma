# Sprint 94 Implementation Report

## Repository transition

PR #24 was reviewed as a narrow two-line correction to the epoch-two
prediction-seal expiry classification, made ready, merged, and its remote
branch deleted. The authoritative merged `main` is
`7cab5e766505090673670d8e3ba545c4cc839f3b`.

The live prospective lane remains unchanged: one completed and scorable event,
reward eligibility below its minimum sample threshold, and epoch two
registered but unsealed while awaiting input finality. No live input request
was executed.

## Historical lab result

The historical lane discovered the canonical Upbit `KRW-BTC` daily snapshot:
312 rows from `1757203200000` through `1784073600000`. Its immutable snapshot
digest is `91fc9425cd92ce18`; its conservative contamination audit digest is
`00321b724d144d4f`. The evidence class is
`PreviouslyConsumedResearchEvidence`, with blind-holdout and authority
eligibility both false.

The expanding-window registration digest is `0dd64fb1535a4701`. The causal
policy derived 225 eligible folds from a 16-row context, horizon one, and 64
minimum past-only training examples.

Protocol replay completed 225 of 225 folds:

- scorable: 216
- neutral excluded: 9
- invalid: 0
- performance claim: none

Expanding-window walk-forward completed 225 of 225 folds:

- scorable: 216
- neutral excluded: 9
- invalid: 0
- prediction-before-reveal audit: passed
- chronology audit: passed
- leakage audit: passed

Aggregate historical research metrics:

| Research participant | Mean Brier | Binary correctness | Brier delta vs constant | Research classification |
| --- | ---: | ---: | ---: | --- |
| HistoricalRawFeatureLogisticV1 | 0.2660945384 | 0.4768518519 | +0.0128285581 | BenchmarkBetterOnResearchReplay |
| HistoricalRawFeatureInteractionLogisticV1 | 0.2873403966 | 0.4953703704 | +0.0340744163 | BenchmarkBetterOnResearchReplay |
| HistoricalTrainingPrevalenceConstantV1 | 0.2532659803 | 0.5092592593 | 0 | MixedResearchEvidence |

These are research-only comparisons, not an independent holdout, not
prospective authority, and not a winner selection. A repeated identical replay
returned replay digest `e4c93c505989d94f` with zero writes and zero duplicate
attempts.

## Safety result

The historical lab finished with zero network requests, transports, credential
reads, live prospective count changes, live participant changes, parameter
updates, normalizer refits, rankings, winner selections, rewards, penalties,
Chair decisions, votes, voice or tier changes, cooldowns, promotions,
quarantines, paper executions, and live executions. The protected live
artifact identity and three-member active roster identity were unchanged.

The older-history backfill plan digest is `53d2e54cf4d9ff5e`. It remains
unexecuted, requires explicit network authorization, permits concurrency one,
and permits zero retries.

Trading simulation remains `BlockedNoFrozenExecutionPolicy`. Official Mamba-3
was not implemented or evaluated. Chair remained inactive.

## Proof boundary

Proven:

- deterministic canonical discovery and immutable semantic snapshotting;
- conservative contamination classification;
- preregistration before folds;
- causal expanding-window construction;
- fold-local fresh training and normalization;
- exactly three prediction seals before target reveal;
- neutral exclusion and finite aggregate metrics;
- manual protobuf reopen verification and atomic persistence;
- deterministic completed-replay reuse;
- zero live authority and zero network activity.

Not proven:

- independent unseen performance;
- future generalization;
- live participant superiority;
- reward effectiveness or Chair learning;
- official Mamba-3 behavior;
- paper- or live-trading readiness.

The next single step is to keep the live epoch-two contract unchanged and wait
for its registered input-finality condition; historical backfill should remain
unexecuted until a separate, explicit owner-authorized sprint.
