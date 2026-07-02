# AI Core Definition

## Purpose

Soma's AI core is a numeric, outcome-driven committee system. A collection of
rules, model-readiness reports, or mock opinions is not by itself a completed AI
core. Runtime decisions remain deterministic, paper-first, and subordinate to
the Risk Governor.

## 1. Policy Delegate

A policy delegate:

- applies fixed numeric rules and thresholds,
- owns a doctrine and style-specific vote,
- may propose, abstain, reduce size, or prefer `NoTrade`,
- may have a mutable configuration edited outside the decision,
- does not learn from outcomes by itself.

The current three active personas are policy delegates.

## 2. Learning Agent

A learning agent must have:

- immutable doctrine,
- versioned mutable numeric policy,
- an agent-specific memory and outcome ledger,
- outcome feedback with provenance,
- bounded candidate updates derived from that feedback,
- validation before any candidate becomes the next stable version.

It cannot mutate its live state during a decision. A score counter or journal
entry alone does not make a learning agent.

## 3. Independent Agent

An independent agent must own all of the following:

- unique doctrine,
- unique policy weights or thresholds,
- unique evaluation horizon,
- unique memory and outcome ledger,
- unique reward/penalty history,
- independent vote or proposal,
- style-specific scoring and calibration.

Sharing market data, features, and safety gates is allowed. Sharing every policy
parameter, memory, and update history is not.

## 4. Chair AI

The Chair is a meta-controller, not a super-investor. It:

- selects speakers and a lead speaker,
- preserves dissent and limits groupthink,
- synthesizes numeric votes into a candidate,
- attributes paper outcomes,
- proposes bounded reward, penalty, voice, cooldown, and tier changes,
- processes owner input as advisory evidence,
- emits stable reason codes and fixed explanations.

The Chair cannot execute, relax a hard safety gate, or override the Risk
Governor. Current active Chair logic handles speaking and synthesis; reward and
promotion governance remains offline/shadow.

## 5. Risk Governor

The Risk Governor:

- is not an investor agent,
- is not trained by agent outcomes,
- owns absolute veto authority,
- defaults to denial when there is no valid proposal,
- checks loss, exposure, edge, confidence, liquidity, data quality, regime, and
  execution-risk constraints,
- may approve only a paper plan in the current system.

Agent and Chair performance cannot weaken its thresholds during a decision.

## 6. Self-Learning

Self-learning means:

```text
versioned paper decision
-> realized or counterfactual outcome
-> attributed numeric feedback
-> sandbox candidate update
-> deterministic replay/backtest
-> calibration and risk comparison
-> promotion gate
-> next stable version or rejection
```

It does not mean runtime LLM reasoning, live weight updates, uncontrolled
parameter mutation, or automatic deployment.

## 7. Carrot And Stick

Reward and penalty are outcome-attributed numeric signals:

- profitable paper outcome after cost can be rewarded,
- avoided loss through `NoTrade` can be rewarded,
- correct risk warnings and calibrated confidence can be rewarded,
- high-confidence losses and overtrading are penalized,
- drawdown contribution and poor liquidity acceptance are penalized,
- doctrine or Risk Governor bypass attempts receive severe penalty and may
  trigger quarantine.

## Existing Contract Mapping

| Required concept | Existing source |
| --- | --- |
| Agent identity and status | `PersonaCard`, `AICommitteeMember` |
| Immutable doctrine | `ImmutableDoctrine`, `DoctrineRule` |
| Mutable policy | `MutablePolicy`, `PersonaMutablePolicy` |
| Proposal/vote | `InvestorVote`, `PersonaVote`, `MemberOpinion` |
| Memory summary | `MemberMemoryState` |
| Outcome ledger | `OutcomeRecord`, member learning journals and experience records |
| Feedback | `PaperOutcomeFeedback`, `PersonaEvaluationInput` |
| Reward/penalty | survival scoring, `MemberScoreUpdate`, Chair shadow contracts |
| Learning state | score/voice/tier plus offline learning journal |
| Sandbox status | shadow/readiness/promotion-gate types |

These concepts span multiple historical type families. Sprint 07 selects
`CanonicalAgentState` in `persona_card.rs` as the aggregate for new
paper-learning work while reusing the existing doctrine, policy, horizon, tier,
and reason types. Historical offline committee types remain adapter inputs.

## Runtime Invariants

- Numeric decisions only.
- `NoTrade` by default.
- Immutable doctrine cannot be learned away.
- No mutation during a market decision.
- All changes are versioned and replayable.
- Sandbox candidates never vote in the active committee.
- Risk Governor remains outside agent learning.
- Owner input never forces a trade.
- Paper validation precedes promotion.
