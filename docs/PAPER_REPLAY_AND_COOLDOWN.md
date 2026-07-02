# Paper Replay And Cooldown

## Scope

`run_3_agent_paper_replay` reuses the existing three-agent learning chain one
finalized episode at a time. It adds deterministic replay control and a
post-episode cooldown transition. It does not add another Chair, Risk Governor,
broker, feedback, or attribution implementation.

The active roster remains:

- `momentum_trend_fast`,
- `value_quality_filter`,
- `cycle_risk_skeptic`.

## Replay Flow

```text
initial canonical states
  -> one existing paper learning chain episode
  -> finalized outcome and feedback versions
  -> optional post-episode cooldown tick and version
  -> stop-condition review
  -> next episode with the resulting canonical states
  -> aggregate learning and attribution summaries
```

Replay rejects an empty sequence, more than the configured maximum, a roster
limit other than three, duplicate episode or decision identities, reversed
timestamps, non-causal outcome finalization, and Risk Governor configuration
changes.

## Cooldown Lifecycle

`AgentVoiceState.cooldown_bars` is the remaining completed-episode count.
Reward/penalty may start cooldown through the existing Chair transition.
An agent with a positive count is unavailable for active speaking and emits an
abstaining vote with `CooldownAgentUnavailable`.

With `CooldownTickMode::PerEpisode`, the count decreases by one only after the
episode has produced a finalized paper outcome. The subtraction saturates at
zero. Reaching zero changes `Cooldown` back to `Active` unless a stronger
disabled, sandbox, observer, or quarantine rule applies.
`CooldownTickMode::Disabled` disables automatic ticks.

Every effective tick creates a deterministic child version and an in-memory
paper-only journal snapshot. The next episode therefore sees the ticked
version, while the decision that just completed used the unchanged input
version.

## Bypass Prevention

- Chair selection receives an abstaining vote for a cooldown agent.
- Owner requests containing cooldown-clear or forced-activation intent are
  rejected with stable reason codes and templates.
- Sandbox candidate metadata is never supplied as replay state.
- Risk Governor configuration and evaluation are independent of cooldown.
- Quarantine is stronger than cooldown and is never cleared by a tick.

## Replay Attribution

`ReplayAttributionSummary` accumulates the existing selected, supported,
opposed, abstained, Risk-aligned, NoTrade, reward, and penalty counts. It adds
`cooldown_skipped_count`, final cooldown, final tier, final voice, and final
status. Missing attribution remains explicit through the existing conservative
reason code.

## Stop Conditions

Replay can stop after an episode that produces:

- a quarantined agent when `stop_on_quarantine` is enabled,
- a Risk Governor emergency stop when `stop_on_emergency_stop` is enabled.

The completed episode, all child versions, and the valid journal remain in the
result. No partial decision state is committed.

## Boundaries

Replay uses synthetic or caller-supplied paper inputs only. It performs no
network IO, real account access, real broker call, order cancellation, runtime
LLM call, or live policy mutation. `PaperBroker` remains the only order
boundary, and explicit fill evidence remains mandatory for executed
performance.

This is not a complete AI system. Durable persistence, long historical market
ingestion, eight-agent activation, online learning, heavy models, and live
execution remain outside this implementation.

## Owner Report Projection

`build_owner_learning_report` reads the completed replay result and projects
its existing numeric summaries into an owner view. It cannot change replay
states or journal entries. Report renderers and review commands operate only
on this immutable projection.
