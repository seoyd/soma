# Minimal investor committee

Sprint 32 adds a **minimal** committee layer: exactly three active personas, Chair v0, and a conservative risk bridge.

## Why three personas first

- three voices are enough to surface disagreement and groupthink without overfitting a fake parliament
- the runtime stays numeric, local, deterministic, and reviewable
- expansion to 6/12/18 is deferred to design review, not implementation

## Active MVP personas

1. `trend_breakout_fast` — momentum/breakout trigger
2. `defensive_value_risk` — downside and no-trade skeptic
3. `cycle_regime_guard` — regime and volatility guard

These names are **archetype labels**, not literal investor reproductions.

## Guardrails

- no runtime LLM
- no broker/order/account/live path
- Risk Governor keeps absolute veto
- yfinance remains research-only
- Upbit committee smoke remains crypto-only

