# Chair Reward And Penalty System

## Current Status

The active `ChairEngine` controls speaker selection, contrarian inclusion,
cluster penalties, groupthink handling, and candidate synthesis. It does not
mutate agent scores, policies, tiers, or voice power during a decision.

The repository has deterministic survival scoring, tier evaluation, offline
score updates, and extensive Chair shadow-governance contracts. Those contracts
produce candidates and explicitly block actual mutation. This document defines
how those pieces should converge; it does not activate learning.

`compute_chair_reward_penalty`, `update_agent_voice_state`, and
`apply_chair_reward_penalty` are pure functions. `apply_paper_feedback_cycle`
connects them to finalized paper outcomes and version snapshots. They return
bounded state changes after paper feedback and never mutate a decision-time
state.

`run_3_agent_paper_learning_loop` invokes the same reward/penalty rules for each
attributed agent after a finalized outcome. The Chair and Risk decisions are
complete before any reward is calculated, so a reward cannot validate a denied
proposal or retroactively change speaker selection.

## Chair Authority

The Chair may propose:

- bounded voice-power increase,
- bounded voice-power decrease,
- temporary cooldown,
- committee-level veto privilege adjustment,
- sandbox-only promotion review,
- demotion review,
- quarantine for severe doctrine or risk violation.

The Chair may not:

- modify immutable doctrine,
- update an active version during a decision,
- approve its own candidate for deployment,
- execute an order,
- override the Risk Governor,
- convert owner pressure into a reward.

## Reward Signals

- Positive paper outcome after fees and slippage.
- Avoided loss from a justified `NoTrade`.
- Correct risk warning or helpful dissent.
- Confidence calibrated to realized outcomes.
- Doctrine-consistent decision.
- Low drawdown contribution.
- Useful independent contribution after correlation adjustment.

## Penalty Signals

- High-confidence loss.
- Excess trade frequency or churn.
- Doctrine violation.
- Ignored risk warning or bypass attempt.
- Large drawdown contribution.
- Acceptance of bad spread or low liquidity.
- Owner-pressure compliance against risk policy.
- Repeated correlated voting without independent evidence.

## Survival-First Score

The current implementation uses this bounded weighted score:

```text
survival_first_score =
  0.22 * drawdown_control
+ 0.18 * risk_efficiency
+ 0.17 * net_expectancy_after_cost
+ 0.15 * calibration
+ 0.12 * regime_fit
+ 0.10 * silence_value
+ 0.06 * doctrine_consistency
- overconfidence_penalty
- overtrade_penalty
- correlation_penalty
- doctrine_violation_penalty
```

The result is clamped to `[0, 1]`. The multiplicative product of all positive
terms is not used because one noisy zero would erase otherwise meaningful
safety evidence.

## Proposed Bounded Actions

| Condition | Candidate action |
| --- | --- |
| Strong validated score with enough samples | Small voice increase |
| Useful `NoTrade` or risk warning | Reward and voice increase candidate |
| Weak score or repeated calibration miss | Voice decrease or cooldown |
| High-confidence loss | Stronger penalty than ordinary loss |
| Overtrade or bad-liquidity acceptance | Penalty and tighter sandbox policy |
| Doctrine violation | Severe penalty; demotion review |
| Risk bypass attempt | Immediate quarantine candidate |

All deltas must be bounded, deterministic, and recorded with before/after
values and reason codes.

The implemented paper formula bounds positive return reward, defensive
`NoTrade` reward, confidence-weighted loss penalty, missed-gain penalty,
drawdown penalty, overtrade cooldown, bad-data proposal penalty, Risk Governor
opposition penalty, and doctrine penalty. A high-confidence loss uses twice the
loss multiplier of a low-confidence loss. Doctrine or risk-bypass violations
produce quarantine.

## Application Lifecycle

```text
paper outcome
-> attribution
-> Chair candidate reward/penalty
-> immutable audit entry
-> sandbox state update
-> replay and counterfactual evaluation
-> Risk Governor invariant check
-> promotion/demotion review
-> next version or rejection
```

An update becomes active only between decision sessions and only as a new
version. A failed safety, calibration, sample-size, or determinism gate leaves
the current version unchanged.

## Cooldown, Promotion, And Quarantine

- Cooldown removes speaking rights temporarily but preserves audit history.
- Promotion requires minimum samples, stable survival score, no doctrine
  violation, and no safety regression.
- Demotion is intentionally easier than promotion.
- Quarantine is outside the normal tier ladder and requires explicit reviewed
  re-entry evidence.
- Veto privilege is style-specific and never equals Risk Governor authority.

## Owner Input

Owner feedback may be recorded as evidence. Following an owner request that is
consistent with evidence is not independently rewarded. Following owner
pressure against a safety decision is a penalty signal.

In the canonical loop, owner advice is reviewed only after independent Chair
and Risk outputs exist. The review can explain or record a paper request, but
it is not an input to reward calculation and cannot bypass a Risk denial.

## Repeated Episode Stability

The multi-episode chain applies the same bounded formula in deterministic
episode order. A first high-confidence loss uses the normal confidence-weighted
penalty. A repeated high-confidence loss, detected from the previous canonical
memory summary, adds a bounded repeat penalty and may trigger cooldown.

The cumulative signal cannot modify Risk Governor rules, immutable doctrine, or
mutable policy. Cooldown affects later speaking rights; doctrine violation
still escalates to quarantine. Reward and penalty totals are reported per agent
without changing a decision already made.

## Replay Cooldown

A cooldown action adds `CooldownStarted` through the existing bounded penalty
transition. During later replay decisions the agent abstains, so Chair cannot
select it as an active speaker. `CooldownTicked` and `CooldownExpired` are
post-episode transitions and are not part of the decision-time Chair action.

## Prospective Learned-Agent Bridge

The learned-agent prospective bridge is a registration and candidate boundary,
not an application path. It may later adapt mature, independently attributed
outcomes to the existing bounded `compute_chair_reward_penalty` inputs; it does
not add a second survival score or a new reward formula.

At this boundary, all current prospective counts may remain zero. No code path
calls `apply_chair_reward_penalty`, `apply_paper_feedback_cycle`, voice update,
tier update, cooldown, promotion, demotion, or quarantine functions. Owner
advisory records, including the prior observation-only review, are excluded
from reward attribution. Any future application needs a separately authorized
Chair episode after candidate computation and its normal immutable audit,
replay, and Risk Governor checks.
