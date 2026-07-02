# Investor League Constitution

## Binding runtime rules

The current paper/runtime Soma Zero path is governed by these rules:

1. Runtime LLM is forbidden.
2. All decisions are numeric-only and deterministic.
3. Default action is `NoTrade`.
4. Risk Governor is above Chair and has absolute veto.
5. Chair is a meta-controller only. It may rank, filter, and shape candidates, but it may not execute.
6. Paper execution only. No real broker path is allowed in this sprint.
7. Only 3 active personas are active in the current restart.
8. No live self-mutation is allowed.
9. Evolution remains sandbox-only and is deferred.

## Active personas

| Persona | Role | Horizon | Core doctrine |
| --- | --- | --- | --- |
| `momentum_trend_fast` | Fast trend delegate | `Intraday` | Never average down, speak only on trend/breakout, cut losses quickly |
| `value_quality_filter` | Defensive value-quality filter | `Position` | No intraday entry calls, reject unknown assets, require margin of safety when fundamentals exist |
| `cycle_risk_skeptic` | Risk and cycle skeptic | `Swing` | Risk first, reject poor risk/reward, reject euphoria chasing, cooldown respected |

## Authority split

- **League** produces numeric votes only.
- **Chair** selects speakers, applies contrarian inclusion, cluster penalty, and groupthink handling.
- **Risk Governor** can still deny any Chair-approved candidate.
- **Paper broker** is the only execution surface.

## Future extension path

The owner direction is to expand carefully from 3 independent agents to 8.
Expansion requires outcome evidence and explicit safety approval. The agents
must retain distinct investor-style doctrines, while Chair manages speaking
rights, promotion/relegation, rewards, and penalties. Owner input remains
advisory; rejection must include reasons.

Any future expansion must preserve:

- immutable doctrine
- numeric-only decisions
- Risk Governor veto
- paper-first rollout
- test-backed activation
