# Paper Learning Ledger

## Scope

The paper learning ledger is a deterministic state summary, not neural
training. It records paper outcomes after a decision and creates a new versioned
agent state without mutating the state used by that decision.

## Outcome Feedback

`build_agent_feedback_from_paper_outcome` converts a finalized
`backtest::OutcomeRecord` into `AgentFeedback`. `FeedbackContext` must confirm
paper-only and finalized status before conversion. `AgentFeedback` records:

- agent, proposal, and outcome identifiers,
- mandatory paper-only status,
- executed paper trade, `NoTrade`, Risk denial, or abstention kind,
- realized net return,
- optional counterfactual return kept separate from realized return,
- avoided-loss and missed-gain values,
- drawdown contribution,
- confidence at decision time,
- doctrine violation,
- correct risk warning,
- correct `NoTrade`,
- overtrade detection,
- stable reason codes.

Cross-agent, non-paper, pending, non-finite, or executed-without-fill-result
feedback is rejected before memory, state, snapshot, or sandbox metadata can be
created.

## Memory Summary

`apply_feedback_to_memory_summary` returns a new summary with:

- total decisions,
- paper trades and `NoTrade` decisions,
- wins and losses,
- avoided losses and missed gains,
- high-confidence misses,
- doctrine violations,
- maximum drawdown contribution,
- last outcome event ID.

The function performs no IO and does not alter the input state.

Only `ExecutedPaperTrade` feedback increments paper trades, wins, losses, or
high-confidence misses. `NoTrade` and Risk denial increment the no-trade count;
abstention records participation without creating a trade. Counterfactual
returns may affect role-based reward or penalty but never become realized wins
or losses.

## Reward And Penalty

`compute_chair_reward_penalty` applies survival-first rules:

- positive net paper return receives a bounded reward,
- avoided loss and correct `NoTrade` receive stronger defensive reward,
- correct risk warnings receive reward,
- losses receive confidence-weighted penalty,
- high-confidence losses use twice the low-confidence multiplier,
- missed gains receive a smaller penalty than avoided-loss reward,
- drawdown contributes a safety penalty,
- overtrading triggers a cooldown-level penalty,
- doctrine or risk-bypass violations trigger quarantine.

Voice change is bounded and clamped when applied. Reward/penalty does not create
an `OrderPlan`, modify a `RiskDecision`, or call a broker.

## Versioned Update

`apply_paper_feedback_cycle`:

1. builds feedback from a finalized paper outcome,
2. checks doctrine,
3. computes Chair reward/penalty,
4. creates a new voice state,
5. updates the memory summary,
6. creates a child version linked to the previous version,
7. creates an `AgentStateSnapshot`,
8. optionally creates sandbox-only candidate metadata.

Doctrine and mutable policy remain byte-for-byte equal unless a future explicit
versioned policy-review operation is added. There is no such operation in this
sprint.

## In-Memory Version Journal

`AgentStateJournal` stores `AgentStateSnapshot` values without file or network
IO. It supports append, latest-by-agent, all snapshots by agent, count, and
version existence checks.

Append rejects duplicate version IDs, non-paper snapshots, state/metadata
mismatches, and sandbox snapshots that remain live-enabled. Snapshots contain
canonical state and reason codes, not credentials, raw provider responses,
private documents, or owner notes.

## No Live Mutation

The state used to make a decision is immutable to this flow. The returned state
may be reviewed after the outcome phase. It cannot retroactively affect the
decision, Chair selection, or Risk Governor result.

This is paper-only learning metadata. It is not online learning, live
self-evolution, or model training.

## End-To-End Integration

`run_3_agent_paper_learning_loop` is the canonical caller for one complete
three-agent cycle. It creates a private in-memory journal, appends each
post-outcome state snapshot through `AgentStateJournal::append_snapshot`, and
returns those snapshots with the updated states.

The loop distributes feedback by `AttributionRecord`: lead selection,
support/opposition, abstention, Risk Governor alignment/opposition, correct
`NoTrade`, and missed gain each receive a stable reason code. Opposing and
abstaining agents do not inherit the selected agent's realized return. This
keeps memory and voice updates agent-specific while avoiding heavy attribution.

If the paper context is absent or not finalized, the canonical loop returns no
feedback, reward/penalty, version snapshot, or sandbox candidate. The original
state vector is returned unchanged.

## Multi-Episode Journal

The learning chain appends every episode snapshot to one
`AgentStateJournal`. It verifies the snapshot parent against the agent state
used by that episode before append, then verifies that each agent's latest
snapshot matches the final chain state.

Episode IDs and numeric decision identities are unique. Risk configuration is
held constant. Counterfactual returns remain separate from realized paper
trades throughout aggregation. Each finalized outcome timestamp must precede
the next episode decision timestamp.

Version transition IDs include a stable hash of the complete normalized
`AgentFeedback`, not only the market decision ID. Replaying the same parent and
decision with different outcome content therefore cannot produce two different
states under one version ID.

## Cooldown Versions

An effective replay cooldown tick is appended as a paper-only child snapshot.
Its parent is the completed episode's final feedback version. The event
identity contains the episode, agent, and remaining cooldown count, and the
latest snapshot must match the replay's final state.
