# Backtest Outcome Engine

## Why Sprint 04 exists

Sprint 04 turns Soma Zero from a single-step paper decision path into a deterministic outcome engine that can evaluate:

- executed trade proposals
- `NoTrade` decisions
- risk-denied candidates
- persona vote attribution
- promotion/relegation inputs built from actual historical outcomes

This sprint does **not** add live trading. It only adds a numeric, replay-based evaluation layer.

## Deterministic replay

The backtest path is built on:

- `Candle`
- `CandleSeries`
- `Timeframe`
- `MarketReplayCursor`

`MarketReplayCursor` only exposes the current candle and a lookback window. Signal generation, persona voting, Chair evaluation, and Risk Governor evaluation run on that past-and-present view only.

## No look-ahead in the signal path

Look-ahead protection is explicit:

- `MockSignalEngine` receives only the current `MarketSnapshot`
- personas receive only current market + signal inputs
- Chair receives only current votes
- Risk Governor receives only current risk state and candidate proposal

Future candles are inspected only inside triple-barrier outcome evaluation.

## Outcome-only future inspection

The only Sprint 04 component allowed to read future candles is the triple-barrier evaluator. That function is responsible for:

- take-profit / stop-loss hit order
- time-barrier expiry
- `NoData` handling when horizon exceeds available candles
- gross vs net return calculation
- excursion tracking

## Cost-aware returns

Sprint 04 adds `CostModel` and separates:

- gross return
- net return after fee / slippage / spread costs

Risk Governor and backtest outcome evaluation now both work with cost-aware edges rather than optimistic raw returns.

## Risk Governor counterfactuals

When Risk Governor denies a candidate:

- no execution occurs
- a hypothetical outcome may still be evaluated
- avoided loss becomes positive defensive attribution
- missed gain becomes small opportunity cost only

This keeps veto behavior measurable without weakening the veto.

## NoTrade scoring

`NoTrade` is scored as a real decision:

- hypothetical stop first => positive avoided-loss score
- hypothetical take-profit first => small missed-gain penalty
- neutral => near zero

That keeps survival-first silence from being misclassified as failure.
