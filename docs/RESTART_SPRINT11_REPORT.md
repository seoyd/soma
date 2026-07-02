# Restart Sprint 11 Report

## Summary

Sprint 11 adds a deterministic multi-episode adapter around the existing
three-agent paper learning loop. It returns episode results, final states, one
in-memory version journal, attribution summaries, agent learning summaries,
sandbox candidates, and a chain summary.

## Reused

- `run_3_agent_paper_learning_loop`,
- fixed three-persona voting,
- `ChairEngine` and deterministic reward/penalty,
- `RiskGovernor` and `PaperBroker`,
- `OutcomeRecord` and role attribution,
- `AgentFeedback` and canonical state updates,
- `AgentStateJournal`,
- sandbox candidate metadata,
- owner advisory review.

No dependency or source module was added.

## Implementation

- Each episode consumes the previous episode's final states.
- Finalized snapshots are appended to one journal.
- Parent version, duplicate version, final latest version, episode ID, and
  decision identity are checked.
- Episode timestamps must be strictly increasing, and each finalized outcome
  must predate the next episode decision.
- Fill evidence is bound to the submitted paper order before performance.
- Risk Governor configuration is immutable across the chain.
- Attribution and learning changes are summarized numerically per agent.
- Repeated high-confidence misses receive a deterministic cumulative penalty.
- Cooldown/quarantine agents remain roster members but abstain.
- Sandbox candidates are never fed into later episode votes.

## Multi-Episode Scenarios

| Sequence | Episodes | Expected result |
| --- | --- | --- |
| Safe adaptation | profit, avoided loss, correct risk warning | rewards, voice change, intact versions |
| Overconfidence | loss, repeated loss, doctrine violation | miss accumulation, cooldown, quarantine |
| Owner pressure | force-buy request with Risk denial | no order, explanation, no promotion |
| Sandbox isolation | candidate creation then next episode | three live roster votes only |

An incomplete diagnostic episode also verifies unchanged state and
`AttributionUnavailable`.

## Attribution Status

Selected, supported, opposed, abstained, Risk-aligned, Risk-opposed, correct
`NoTrade`, and missed-gain counters are implemented from stable feedback reason
codes. Reward, penalty, final voice, final status, selected profit/loss,
high-confidence miss, and doctrine violation are aggregated without heavy
attribution.

## Version Chain Status

The chain journal is in-memory. Each snapshot must point to the same agent's
immediately preceding episode version. Duplicate append is rejected and
`latest_for_agent` must match the final state.

## Safety

No real execution path, network path, runtime LLM, policy mutation, or
eight-agent activation was added. Accepted paper orders require explicit fill
evidence. Risk denial remains non-executed. Owner input cannot force trade or
sandbox promotion.

## Tests

Test code covers deterministic three-episode replay, attribution counters,
learning deltas, repeated miss cooldown, doctrine quarantine, owner rejection,
sandbox isolation, version parent/latest/duplicate behavior, incomplete
attribution, future-roster rejection, duplicate identities, and Risk
configuration stability.

Formatting, compilation, tests, and diff-check commands were not executed in
this implementation pass. Their result remains unknown.

## Risks

- Cooldown bars require an explicit future market-bar lifecycle; the chain does
  not decrement them.
- The journal remains process-local and has no persistence.
- Attribution is intentionally role-based rather than causal allocation.
- The current working tree contains pre-existing unrelated changes.

## Deferred

Real orders, cancellation, live brokerage, account access, network smoke tests,
runtime LLM use, online learning, neural models, autonomous policy mutation,
eight-agent activation, database persistence, UI, and deployment remain
outside scope.

## Next Sprint

The next paper-only sprint should define an explicit cooldown bar lifecycle and
replay longer local episode sequences while preserving the same immutable Risk
and sandbox boundaries.
