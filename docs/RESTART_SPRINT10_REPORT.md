# Restart Sprint 10 Report

## Summary

Sprint 10 connects the existing three-persona committee and paper learning
primitives through `run_3_agent_paper_learning_loop`. The result contains
votes, proposals, Chair output, Risk decision, optional paper order and
outcome, feedback, rewards/penalties, updated states, version snapshots,
sandbox candidates, owner explanation, reason codes, and a compact report.

## Reused Components

- existing three persona delegates and fixed-roster vote helper,
- `ChairEngine` and its numeric trade proposal,
- `RiskGovernor` and paper-only `OrderPlan`,
- `PaperBroker`,
- `OutcomeRecord` and attribution roles,
- `AgentFeedback`, `CanonicalAgentState`, and memory update,
- deterministic Chair reward/penalty and `AgentStateJournal`,
- `SandboxPromotionCandidate`,
- owner trade-request review and explanation templates.

No new dependency or source module was added.

## Role Review

- Product and scope review kept the work to the three-agent paper loop.
- Learning and league review distributed feedback to all fixed delegates.
- Chair review kept reward/penalty after finalized outcomes.
- Risk review kept the governor as the last authority before `PaperBroker`.
- Paper review used deterministic supplied results and counterfactuals only.
- Rust review kept the implementation in the existing learning-state module.
- QA coverage was written for scenarios and invariants.
- Release review records unverified commands and deferred production work.

## Scenarios Implemented

| Scenario | Expected loop behavior |
| --- | --- |
| Profitable paper trade | selected agent win, positive reward/voice, child version |
| High-confidence loss | stronger penalty than lower confidence, miss counter |
| `NoTrade` avoids loss | defensive reward and avoided-loss counter |
| Risk denial | no paper order, denial and attribution reasons retained |
| Doctrine violation | maximum penalty/quarantine, no live candidate |
| Owner force trade | Risk denial stands, stable advisory explanation |

## Feedback And Versioning

The adapter emits explicit attribution reasons for selected, supported,
opposed, abstained, risk-aligned, risk-opposed, correct `NoTrade`, and missed
gain outcomes. Feedback updates only the matching agent.

Finalized outcomes create one child state and in-memory snapshot per feedback
recipient. Original decision state, immutable doctrine, and mutable policy
remain unchanged. Pending outcomes create no state update. Accepted paper
orders require explicit finalized fill evidence before realized feedback.
`NoTrade`, Risk denial, and abstention cannot increment realized trade wins or
losses.

## Sandbox Safety

Candidates are optional, paper-derived review metadata. They remain
sandbox-only, cannot vote, cannot affect Risk Governor or active state, and
cannot promote themselves. Quarantined agents do not produce candidates.

## Verification

Test code was added for six scenarios, deterministic replay, post-outcome
timing, fixed roster rejection, parent-version linkage, immutable doctrine and
policy, paper-only order behavior, and zero live calls.

Formatting, compilation, test, and diff-check commands were intentionally not
executed in this implementation pass. Their status is therefore unverified.

## Risks

- Synthetic outcome context assumes a trusted finalized paper evaluator.
- Attribution is role-based and deliberately avoids complex allocation.
- The journal is in-memory; restart persistence is absent.
- Only the fixed three-agent roster is accepted.

## Deferred

Real orders, cancellation, live brokerage, live account access, network smoke
tests, runtime LLM use, online learning, neural model work, autonomous policy
mutation, eight-agent activation, database persistence, UI, and deployment are
not implemented.

## Next Sprint

The next paper-only sprint should connect a multi-cycle backtest outcome stream
to this adapter, validate version chaining across repeated finalized events,
and measure attribution stability without changing live policy.
