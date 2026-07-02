# Restart Sprint 12 Report

## Summary

Sprint 12 adds deterministic long paper replay and a post-episode cooldown
lifecycle around the existing three-agent learning chain.

## Reused

- `run_3_agent_paper_learning_chain`,
- the single-episode Chair, Risk Governor, and `PaperBroker` path,
- canonical agent state and voice fields,
- feedback, reward/penalty, attribution, journal, and sandbox types,
- owner advisory review and stable explanation templates.

## Implementation

- Added replay input, configuration, result, error, and attribution types.
- Added `run_3_agent_paper_replay`.
- Added per-episode or disabled cooldown tick modes.
- Added deterministic cooldown child versions and journal snapshots.
- Added quarantine and emergency-stop replay boundaries.
- Added owner and Chair cooldown-bypass rejection reasons.
- Kept the active roster fixed at three.

## Replay Scenarios

| Scenario | Sequence | Expected state |
| --- | --- | --- |
| Stable learning | profit, avoided loss, Risk warning, profit | deterministic growth without bypass |
| Repeated loss | high-confidence losses then doctrine violation | lower voice, cooldown, possible quarantine |
| Cooldown lifecycle | cooldown episodes until zero | abstain, tick, expire, active |
| Risk denial | denial and owner pressure | no execution or owner override |
| Sandbox isolation | candidate then later episode | three canonical voters only |

## Cooldown Behavior

Cooldown is represented by `cooldown_bars`. A completed finalized paper episode
can reduce it by one. The decision uses the pre-tick state; the tick is a
separate post-episode child version. Counts saturate at zero. Quarantine,
disabled, and sandbox states are not cleared by ticking.

## Attribution And Version Status

Replay attribution aggregates existing role-based counts and records cooldown
skips plus final cooldown, tier, status, and voice. Every feedback update and
effective cooldown tick has a deterministic parent-linked snapshot. The final
journal snapshot must match each returned final state.

## Tests

Test code covers deterministic four-episode replay, cooldown skip/tick/expiry,
quarantine stop, emergency stop, owner and Chair bypass rejection, final
journal identity, immutable doctrine/policy, and fixed three-agent operation.

Formatting, compilation, tests, and diff-check commands were not executed in
this implementation pass. Their result remains unknown.

## Safety

No real broker, real order, cancellation, live network, runtime LLM, online
learning, heavy model, full eight-agent roster, or live mutation path was
added. Risk Governor remains independent and sandbox candidates remain
non-voting metadata.

## Risks

- The journal remains in memory.
- Cooldown duration is completed-episode based, not exchange-bar based.
- Attribution remains role-based rather than causal allocation.
- Verification commands have not been executed.

## Deferred

Durable replay persistence, historical dataset ingestion, exchange-bar
cooldown policy, eight-agent activation, neural training, and real execution
remain deferred.

## Next Sprint

The next paper-only sprint should verify the accumulated implementation and
then add bounded replay persistence or longer local fixture sequences without
changing broker or Risk Governor authority.
