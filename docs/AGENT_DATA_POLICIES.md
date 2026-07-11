# Agent Data Policies

The current three agents use configurable policies and a configured runtime
universe. No production policy contains a symbol list or provider endpoint.

| Agent | Required evidence | Optional evidence | Primary purpose |
| --- | --- | --- | --- |
| Momentum/Trend | Daily OHLCV | adjusted prices, volatility, liquidity | price, return, trend, volume, volatility |
| Value/Quality | adjusted daily OHLCV, quarterly fundamentals | valuation metrics, corporate actions | valuation, quality, longer-horizon evidence |
| Cycle/Risk | market index, volatility | breadth, liquidity, macro data | drawdown, regime, defensive risk evidence |

Required evidence that cannot be acquired stays visible and produces an
Abstain or NoTrade-compatible gate. Missing optional evidence remains in the
bundle metadata and cannot silently become another agent's signal.

Base snapshots can be shared through the broker, but each bundle retains its
own requested datasets, snapshot IDs, missing evidence, freshness state, and
provenance receipts. A policy change changes the generated intent and therefore
the acquisition plan; no agent is declared superior by configuration.

The policy set intentionally covers only the three active agents. Future agent
expansion requires separate policy and evidence validation before activation.
