# Sprint 100 Event-Two Close Report

## Repository and historical boundary

PR #30 was reviewed as post-result diagnostic-only work and merged into
authoritative `main` at `694547df259509cc4c6bc1937dc329b7b98dcb77`.
The merged diagnostic report and challenger requirements reopened without
model or holdout execution authority. Q1 through Q4 remain
`NotEligibleForSealedHoldout`.

The sealed historical holdout stayed closed: label reads, prediction reads,
metric reads, and execution modes were all zero. Before the event-two
operation, 176 protected live artifacts had aggregate identity
`f792af1c1ee6840d`. Event-one chain identity was `5a69bc91d713478d`;
event-two sealed-chain identity was `c5322c8865b35f97`; historical,
Qualified-Six replay, and diagnostic-store identities were respectively
`f92b9d39d5ce0993`, `2c4b1c2e3af6a6f1`, and `21a3c27d4aa7d647`.
These identities and active roster `27bcd0d843c107fa` remained unchanged.

## Preflight and acquisition

Event two was sealed before outcome access under prediction capsule
`f0fc2d24e1c920e4`, journal `ed46f8a8b3f4f806`, and locked outcome plan
`ae798b355d36bb74`. Actual UTC was `2026-07-27T03:58:12Z`, after finality at
`2026-07-27T00:00:00Z`.

Two text statuses, two JSON statuses, and text and JSON dry-runs matched
exactly. They reported `ReadyForOutcomeAcquisition`, provider Upbit, symbol
`KRW-BTC`, daily cadence, the single locked timestamp, request fingerprint
`a04855c108ad6a33`, maximum requests one, retries zero, concurrency one, prior
attempts zero, and all work and authority counters zero.

Exactly one public outcome request was made. The HTTP response succeeded and
the bounded response contained exactly one finalized row at the registered
timestamp. No health, time, market-list, ticker, pagination, redirect,
fallback, context, participant-specific, or retry request was made.
Successful receipt `738d106d50a89df6` and sealed outcome capsule
`c67ab84dffddf3eb` were persisted. Acquisition ended at
`ReadyForOutcomeOpening` with labels and probabilities still closed.

## Separate opening and ledger

Opening ran under a separate network-free owner authorization,
`e095a9064ad9711a`. The frozen horizon-one label policy classified event two
as `ScorableBinaryOutcome`. Opening bundle `6c7e1d55a1da912b` bound all three
sealed predictions to that label.

Public participant results are:

- `RawFeatureLogisticV4`: `Scored`
- `RawFeatureInteractionLogisticV4`: `Scored`
- `TrainingPrevalenceConstantV4`: `Scored`

No numeric probability, Brier contribution, correctness value, return, OHLCV,
parameter, normalizer value, or feature value is published.

Event-two ledger entry `9a8711351dbcb931` was appended without rewriting the
event-one ledger. The complete ledger derives two completed and two scorable
events. Eligibility receipt `c94ac1fe15dcfb0c` remains
`IneligibleMinimumSamples`.

Winner selections, rankings, reward and penalty applications, Chair model
executions, Chair learning, Chair decisions, committee votes, voice and tier
changes, cooldowns, promotions, quarantines, paper executions, live
executions, and trading actions are all zero.

Additive pause `e616e1deb5e935c9` preserves the prior pause and records
`PausedAfterCompletedEpochTwo`. No epoch-three registration, input plan,
request, or prediction exists. The next research priority is micro
feature/label challenger design; the live lane remains manually paused.

## Replay and verification

Completed status and dry-run replays matched exactly on
`OutcomeAlreadyOpened`. Both returned before transport construction, raw
outcome loading, private prediction access, label derivation, evaluation,
ledger append, eligibility derivation, or writes. Acquisition and opening
authority replays were also verified as zero-work completed replays.

All 49 event-two close tests passed under Default and Metal. Formatting,
workspace checks, the complete Default and Metal workspaces, and the existing
prospective and historical test families were verified sequentially with one
Rust process at a time. Full Default passed `1,153 + 404 + 12`; full Metal
passed `1,154 + 404 + 12`.

This result verifies immutable acquisition, separated opening, append-only
evaluation, eligibility gating, historical isolation, and zero execution
authority. It does not establish model superiority, reward effectiveness,
Chair learning, promotion readiness, official Mamba-3 behavior, or trading
readiness.
