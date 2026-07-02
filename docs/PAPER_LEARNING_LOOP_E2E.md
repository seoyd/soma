# Three-Agent Paper Learning Loop

## Scope

`run_3_agent_paper_learning_loop` is the canonical deterministic learning-loop
adapter for the current three-agent roster:

1. `momentum_trend_fast`
2. `value_quality_filter`
3. `cycle_risk_skeptic`

Any missing, duplicate, extra, or future-placeholder agent makes the call fail.
An agent without current speaking rights is represented by an abstaining vote
and cannot regain those rights inside the decision.

## Flow

```text
MarketSnapshot + SignalOutput + immutable input state
  -> three persona votes
  -> ChairEngine selection and TradeProposal composition
  -> RiskGovernor review
  -> optional PaperBroker paper order
  -> finalized synthetic/backtest OutcomeRecord
  -> per-agent attribution and AgentFeedback
  -> deterministic ChairRewardPenalty
  -> child CanonicalAgentState
  -> in-memory AgentStateJournal snapshot
  -> optional sandbox-only candidate
  -> PaperLearningLoopResult + PaperLearningLoopReport
```

The decision phase reads cloned input states. Feedback, reward/penalty, memory,
and version updates begin only when `PaperOutcomeContext.outcome_finalized` is
true. A pending or absent context returns no feedback, snapshots, or candidates
and preserves all input states.

## Modules Used

- `league::{momentum_trend_fast, value_quality_filter, cycle_risk_skeptic}`:
  numeric persona votes.
- `league::default_league_votes`: fixed three-member vote collection.
- `chair::ChairEngine`: speaker selection, committee score, and proposal.
- `risk::RiskGovernor`: absolute final review and paper order plan.
- `paper::PaperBroker`: the only order boundary; live execution is unsupported.
- `backtest::{OutcomeRecord, AttributionRecord, TripleBarrierResult}`:
  finalized paper and counterfactual result representation.
- `league::persona_card`: canonical state, feedback, reward/penalty, journal,
  sandbox metadata, and the E2E adapter.
- `owner::review_owner_trade_request`: stable advisory rejection explanation.

## Outcome And Attribution

An approved risk decision may create one accepted `paper_only` order. The
supplied deterministic paper context must explicitly mark
`FilledPaperOrder`; only that fill evidence materializes an executed
`OutcomeRecord`. An accepted order without fill evidence cannot create
realized feedback. `PaperFillEvidence` must carry a non-empty fill ID and match
the submitted paper order ID, symbol, paper-only flag, and a fill timestamp not
earlier than submission or later than `finalized_at_timestamp_ms`.
Risk denial and `NoTrade` create no order; a finalized hypothetical return may
still support defensive or opportunity-cost feedback.

Every roster member receives role-based feedback. The loop distinguishes the
lead proposal, supporting and opposing votes, abstention, Risk Governor
alignment/opposition, correct `NoTrade`, and missed gain. It does not use
Shapley values or expensive counterfactual search.

## Safety And State Timing

- `NoTrade` remains the default when Chair or Risk does not approve.
- Risk denial never submits a paper order.
- Owner input is evaluated after independent Chair and Risk decisions.
- Immutable doctrine and mutable policy are copied unchanged.
- Voice, memory, status, and version changes are outcome feedback only.
- Each child version preserves its parent and paper feedback event.
- The journal is in-memory and rejects duplicate, non-paper, inconsistent, or
  live-enabled sandbox snapshots.
- Sandbox candidates are metadata with `sandbox_only = true`, cannot vote or
  affect a live decision, and never start promoted.

## What Can Be Exercised

Deterministic scenarios cover a profitable paper trade, high-confidence loss,
`NoTrade` avoiding a loss, Risk Governor denial, doctrine violation, and an
owner force-trade request rejected by Risk. Repeating the same complete input
must produce an equal result.

## Boundaries

This path performs no random sampling, wall-clock reads, file IO, database IO,
network calls, runtime LLM calls, real broker calls, order cancellation, or
live order placement. `PaperBroker::supports_live_execution()` is false and its
live call count remains zero.

This is not a complete AI system. It is deterministic paper feedback over
three hand-coded delegates. Neural training, online learning, autonomous policy
search, full eight-agent operation, production persistence, and live execution
remain outside this path.

## Multi-Episode Use

`run_3_agent_paper_learning_chain` is the canonical multi-episode caller. It
does not replace or duplicate this loop. It supplies each episode with the
previous episode's final state, appends returned snapshots to one journal, and
builds numeric attribution and learning summaries.

Cooldown or quarantined roster members remain among the fixed three but are
converted to abstaining votes. Sandbox candidates are returned separately and
never become episode input.

Long replay continues to call this exact loop through the learning chain.
Cooldown agents emit abstaining votes before Chair evaluation; their counters
can change only after a finalized episode returns.
