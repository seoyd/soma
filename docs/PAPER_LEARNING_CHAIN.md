# Three-Agent Paper Learning Chain

## Scope

`run_3_agent_paper_learning_chain` connects multiple finalized calls to
`run_3_agent_paper_learning_loop`. It keeps the fixed roster:

1. `momentum_trend_fast`
2. `value_quality_filter`
3. `cycle_risk_skeptic`

No future placeholder or sandbox candidate can enter this roster.

## Chain Flow

```text
initial CanonicalAgentState[3]
  -> episode 1 decision from immutable input state
  -> finalized paper outcome
  -> feedback + reward/penalty + child versions
  -> episode 2 receives episode 1 final states
  -> finalized paper outcome
  -> feedback + reward/penalty + child versions
  -> episode N
  -> final states + journal + numeric summaries
```

Each episode reuses the existing single-episode Chair, Risk Governor,
`PaperBroker`, outcome, feedback, and sandbox path. The chain overwrites only
the episode input's state vector with the previous episode's final states.
Market, signal, risk snapshot, owner advisory, and finalized paper evidence
remain episode-specific.

## State Transition Timing

The state used by an episode is cloned before votes are produced. Voice,
memory, tier, status, and version updates occur only after the episode has a
finalized paper outcome. An incomplete episode is rejected by default. A
diagnostic configuration may retain it, but it produces no feedback, snapshot,
candidate, or state change.

Cooldown and quarantine remove current speaking rights. The agent remains one
of the three roster members and emits an abstaining vote in later episodes.
Cooldown bars are not decremented by the chain because an episode is not a
wall-clock or market-bar clock.

## Version Chain

Every returned snapshot is appended to one in-memory `AgentStateJournal`.
Before append, the chain checks that:

- the snapshot parent equals that agent's input version for the episode,
- the version ID is not already present,
- paper-only and sandbox/live consistency checks pass,
- the latest journal snapshot equals the final returned state.

Episode IDs and decision identities (`symbol + timestamp`) must be unique.
Episode timestamps must also increase in input order, preventing a later
learned state from being applied to an earlier market decision. A finalized
episode's `finalized_at_timestamp_ms` must be earlier than the next episode's
market timestamp, so future outcome knowledge cannot enter an earlier state.
With `N` finalized episodes and three feedback recipients, the journal contains
`3 * N` ordered child snapshots.

## Attribution Summary

`AgentAttributionSummary` counts existing role reason codes:

- Chair-selected speaker,
- supported or opposed final decision,
- abstention,
- Risk Governor veto alignment or opposition,
- correct `NoTrade`,
- `NoTrade` missed gain,
- profitable or losing selected outcome,
- high-confidence miss and doctrine violation.

It also sums bounded Chair reward and penalty values and records final voice
and status. `AttributionUnavailable` is emitted when a diagnostic incomplete
episode has no feedback or no role can be determined. Attribution is
role-based; no Shapley or complex counterfactual engine is used.

## Agent Learning Summary

`AgentLearningSummary` compares initial and final state numerically. It reports
version, voice, tier, status, memory deltas, candidate count, cooldown, and
quarantine for each agent. `LearningChainSummary` reports episode-level paper
trades, `NoTrade`, Risk denial, candidate count, and safety violations.

Repeated high-confidence losses use prior miss memory. The second repeated miss
receives an additional deterministic penalty and may trigger cooldown. Doctrine
violation remains a quarantine-level event.

## Safety Invariants

- Risk Governor configuration must be identical for every episode.
- A denied decision cannot have a paper order or executed outcome.
- Accepted paper orders need explicit finalized fill evidence before realized
  performance exists. Evidence is matched to paper order ID, symbol, paper-only
  status, and fill time. Fill time cannot exceed the outcome finalization time.
- Immutable doctrine and mutable policy must remain unchanged.
- Original episode state cannot be mutated during decision.
- Owner review cannot force a trade or promotion.
- Sandbox candidates remain metadata and are never copied into later votes.
- `PaperBroker` remains the only order boundary and reports no live support.

## Boundaries

The chain performs no random sampling, wall-clock read, file IO, database IO,
network call, runtime LLM call, real broker call, cancellation, or live order
placement. State changes are versioned paper feedback between episodes, not
live self-mutation.

This is not a complete AI system. It is a deterministic three-agent paper
learning ledger. Online training, autonomous policy search, production
persistence, full eight-agent operation, and live execution remain deferred.

## Long Replay

`run_3_agent_paper_replay` is the canonical long-sequence adapter. It invokes
this chain with one finalized episode at a time so an optional cooldown tick
can be recorded as a separate post-episode child version before the next
decision. This chain remains the canonical learning implementation.
