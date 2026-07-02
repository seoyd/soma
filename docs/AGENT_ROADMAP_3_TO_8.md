# Agent Roadmap: 3 To 8

## Current Three-Agent MVP

The current three are deterministic policy delegates. They provide the minimum
committee diversity needed to evaluate the pipeline, but they are not yet
outcome-trained learning agents.

### 1. `momentum_trend_fast`

- Style: trend and momentum.
- Horizon: intraday/short swing.
- Role: generate breakout and trend candidates.
- Rewards: cost-adjusted trend capture, calibrated confidence, disciplined
  exits, and silence outside its regime.
- Penalties: overtrading, averaging down, poor-liquidity entry, and
  high-confidence losses.

### 2. `value_quality_filter`

- Style: defensive quality and value.
- Horizon: position/longer horizon.
- Role: filter universe and cap exposure; not an intraday entry generator.
- Rewards: rejecting weak evidence, protecting margin of safety, and avoiding
  low-quality assets.
- Penalties: accepting unscorable assets, intraday entry calls, or excessive
  exposure without fundamentals.

### 3. `cycle_risk_skeptic`

- Style: cycle, regime, and capital protection.
- Horizon: swing/multi-horizon risk.
- Role: veto or reduce size under poor risk/reward, volatility, or euphoria.
- Rewards: avoided loss, timely risk warnings, and controlled drawdown.
- Penalties: missed material risk, weak veto calibration, or following pressure
  against risk policy.

## MVP Completion Gates

The three-agent MVP is complete only after each delegate has a persisted
versioned policy, private outcome ledger, deterministic sandbox update, and
validated promotion process. A canonical in-memory state and sandbox metadata
now exist, but persistence and promotion evaluation are still missing. Current
fixed policies satisfy committee plumbing, not the full learning definition.

## Future Eight-Agent Target

`v1` means the current three-policy foundation. `v2` means a paper-only
candidate after canonical agent-state and sandbox evaluation exist. `v3` means
later expansion after cross-market evidence and promotion gates.

| Agent | Doctrine | Horizon | Market fit / allowed assets | Output | Reward | Penalty | Veto | Activation |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Quality/Value | Margin of safety, durable quality, avoid unknowns | Position/long | Validated US/KR equities; crypto excluded without comparable fundamentals | Universe filter, exposure cap, approve/reject | Drawdown control, quality selection, patient silence | Unknown assets, weak fundamentals, style drift | No; may hard-reject unscorable input | Active policy in v1; learning candidate v2 |
| Growth/Story | Evidence-backed growth with valuation discipline | Swing/position | Liquid US/KR growth equities | Thesis score, candidate, evidence request | Growth realization with calibrated expectations | Narrative-only calls, valuation neglect, evidence gaps | No | Research-only v1; candidate v2 |
| Momentum/Breakout | Trade strength, cut loss, never average down | Intraday/short swing | Liquid US/KR equities and BTC where spread/volume gates pass | Entry/exit candidate, `NoTrade` | Net trend capture, exit discipline | Overtrade, averaging down, high-confidence loss | No | Active policy in v1; learning candidate v2 |
| Trend/System | Follow persistent moves with systematic exits | Swing/multi-day | Validated liquid equities and BTC | Direction, stop, horizon, size suggestion | Regime-consistent expectancy | Churn, late reversal, rule deviation | No | Deferred v1; candidate v2 |
| Macro/Regime | Regime first; exposure follows liquidity and cycle | Multi-week/position | Market indices, broad regime data, BTC; single assets only through regime overlay | Regime label, exposure multiplier, hold gate | Correct regime transition and risk reduction | Slow transition, unsupported macro claim | May cap exposure, not bypass Risk Governor | Deferred v1; candidate v2 |
| Reflexivity/Asymmetry | Seek convex payoff and detect feedback loops | Event/swing | Highly liquid equities and BTC with validated event data | Asymmetry score, scenario set, abstain | Positive asymmetry after cost | Story chasing, poor liquidity, unbounded downside | No | Research-only through v2; candidate v3 |
| Quant/Statistical Edge | Repeatable calibrated edge after costs | Intraday/swing | Symbols with sufficient clean historical data | Probability distribution, expected edge, calibration | Out-of-sample calibration and net expectancy | Leakage, instability, turnover, cost miss | No | Deferred through v2; candidate v3 |
| Tail-Risk/Skeptic | Survival first; distrust crowded confidence | All horizons | All supported assets and aggregate portfolio | Veto, reduce size, cooldown, `NoTrade` | Avoided loss, correct tail warning | False-negative tail risk, uncalibrated permanent veto | Yes within committee; Risk Governor remains final | Active policy in v1; learning candidate v2 |

## Expansion Rules

- Do not activate an agent to fill a seat.
- New agents begin non-voting and shadow-only.
- Validate unique contribution, not just standalone accuracy.
- Penalize correlation with existing agents.
- Require sufficient samples across the agent's declared horizon and regimes.
- Promote one agent at a time.
- Keep at most one new activation under evaluation per gate cycle.
- Preserve the three-agent fallback until the expanded committee proves safer.
