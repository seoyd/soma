# Paper Outcome Feedback Adapter

## Canonical Input And Output

The canonical adapter input is `backtest::OutcomeRecord`. It already contains
the completed paper decision ID, execution result, Risk Governor denial,
`NoTrade` counterfactual, triple-barrier result, return, drawdown evidence,
attribution, and reason codes.

The canonical output is `league::AgentFeedback`. The adapter is
`build_agent_feedback_from_paper_outcome` in `league::persona_card`.

`FeedbackContext` supplies only facts not represented by `OutcomeRecord`:

- confirmation that the source is paper-only,
- confirmation that outcome production is finalized,
- an externally detected doctrine violation,
- an externally detected overtrade event.

## Existing Outcome Inventory

| Concept | Existing type | Use |
| --- | --- | --- |
| Paper order | `core::PaperOrder` | PaperBroker accepted-order record |
| Paper ledger | `paper::Ledger` | In-memory paper orders and audit events |
| Decision | `backtest::DecisionRecord` | Signal, votes, Chair, Risk Governor, proposal, paper order ID |
| Canonical completed outcome | `backtest::OutcomeRecord` | Adapter input |
| Barrier result | `backtest::TripleBarrierResult` | Net return and adverse excursion |
| Barrier classification | `backtest::TripleBarrierOutcome` | Completed versus `NoData` |
| NoTrade evaluation | `backtest::NoTradeEvaluation` | Avoided loss and missed gain |
| Attribution | `backtest::AttributionRecord` | Agent-specific Risk Governor alignment |
| Shadow outcome | `backtest::ShadowOutcomeRecord` | Pending-state guard and offline evidence |

## Existing Feedback And State Inventory

- `AgentFeedback` is the canonical numeric feedback.
- `CanonicalAgentState` is the canonical agent state.
- `AgentMemorySummary` is the bounded aggregate memory.
- `ChairRewardPenalty` is the deterministic Chair update instruction.
- `AgentVersion` records parent and feedback lineage.
- `AgentStateSnapshot` records the versioned state.
- `AgentStateJournal` is the in-memory paper-only journal.
- `SandboxPromotionCandidate` is metadata with no live effect.

## Field Mapping

| Feedback field | Source or rule |
| --- | --- |
| `agent_id` | canonical state after proposal identity validation |
| `proposal_id` | `AgentProposal::proposal_id` |
| `outcome_id` | `OutcomeRecord::decision_id` |
| `paper_only` | mandatory true `FeedbackContext` assertion |
| `realized_net_return` | `realized_net_return_pct`, or zero for missing executed return |
| `avoided_loss_score` | non-negative outcome value |
| `missed_gain_penalty` | absolute outcome value because the simulator stores cost as negative |
| `drawdown_contribution` | non-negative maximum adverse excursion |
| `confidence_at_decision` | bounded agent proposal confidence |
| `doctrine_violation` | context or doctrine/risk-bypass proposal reason |
| `risk_warning_correct` | Risk Governor denial plus aligned attribution or `NoTrade` stance |
| `no_trade_correct` | denied/NoTrade outcome with positive avoided loss |
| `overtrade` | feedback context |
| `reason_codes` | stable union plus explicit adapter classifications |

An executed outcome without a barrier result uses zero return and records
`FeedbackMissingReturn`. This is conservative and does not fabricate profit.
`NoData`, pending shadow evaluation, an empty outcome ID, cross-agent input,
proposal/outcome symbol or horizon mismatch, inconsistent terminal
classification, non-paper input, and non-finite numeric input are rejected.

## Safety Boundary

The adapter is deterministic and performs no IO. It does not submit orders,
alter broker data, change Risk Governor rules, or update the state used by a
decision. It accepts completed paper evidence only.

No fill-model expansion, heavy attribution, model training, online learning,
live trading, or real broker integration is included.
