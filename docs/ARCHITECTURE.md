# Soma Zero Architecture

## Runtime path

The **Soma Zero v0** runtime path is now the root `soma-zero` crate, not a nested workspace member.

The active numeric-only loop is:

1. `MarketSnapshot`
2. deterministic feature derivation
3. `MockSignalEngine`
4. 3 numeric investor delegates
   - `momentum_trend_fast`
   - `value_quality_filter`
   - `cycle_risk_skeptic`
5. Chair v0
6. Risk Governor v0
7. paper-only broker
8. deterministic audit events

## Numeric-only decision model

- No runtime LLM is used in `soma-zero`.
- No stochastic sampling is used in the decision path.
- Same input should produce the same output.
- Wall-clock time is not consulted by the decision functions; timestamps are passed in as data.

## Chair vs Risk Governor

The separation is strict:

- **Chair** selects speakers, applies contrarian inclusion, applies cluster penalties, and emits a candidate decision.
- **Risk Governor** is the final authority.
- **Chair cannot execute trades and cannot bypass risk.**
- In full-auto mode, `RequireConfirm` degrades to `NoTrade`.

## Risk posture

The system is intentionally survival-first:

- default action is `NoTrade`
- Risk Governor default action is `Deny`
- positive expected edge is required
- stop loss and take profit are required
- poor data quality, low confidence, wide spread, unknown regime, or daily-loss breaches block trading
- paper execution only

## MVP scope

Implemented or normalized for v0:

- core types
- structured `ReasonCode`
- deterministic audit events
- mock signal model
- 3-persona investor league with explicit `PersonaCard` doctrine/policy separation
- Chair v0
- Risk Governor v0
- paper broker
- promotion/relegation scoring helpers
- deterministic tests

## Investor League governance

Sprint 03 keeps the League small and mechanical:

- only `momentum_trend_fast`, `value_quality_filter`, and `cycle_risk_skeptic` are active
- each delegate has:
  - immutable doctrine
  - mutable policy knobs
  - evaluation profile
  - bounded voice power
- doctrine violations and severe events can demote or quarantine a persona
- shadow evaluation exists only as scaffolding and does not affect live execution

## Deferred stack

The following remain outside the Soma Zero v0 runtime path.

Still active in the legacy workspace:

- none

Quarantined by Sprint 03:

- `soma-train`
- `soma-orchestrate`
- `soma-bench`
- `soma-prop`
- `soma-replay`
- `soma-serve`
- `soma-canary`
- `soma-soak`
- `soma-cli`
- `soma-release`
- `soma-online`
- `soma-adapt`
- `soma-ssm`
- `soma-gdn`
- `soma-attn`
- `soma-mor`
- `soma-memory`

Legacy crates are now either **deferred conceptually** or **removed from the active repository**.

## Future extension points

- **6 / 12 / 18 investor league** can extend `src/league/`
- **Mamba3Fin-lite** can later replace or augment `MockSignalEngine` behind the signal interface
- **sandbox evolution** can later attach to scoring and persona-tier transitions, but must remain offline
