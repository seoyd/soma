# Triple-Barrier Labeling

## Overview

Sprint 04 replaces the old single-path barrier helper with a cost-aware triple-barrier evaluator over `CandleSeries`.

## Barriers

`TripleBarrierConfig` defines:

- `take_profit_pct`
- `stop_loss_pct`
- `horizon_bars`
- `fee_bps`
- `slippage_bps`
- `side`
- `use_high_low_intrabar`

The evaluator returns:

- `TripleBarrierOutcome`: `Win`, `Loss`, `Neutral`, `NoData`
- `BarrierHit`: `TakeProfit`, `StopLoss`, `TimeExpired`, `NoData`
- `TripleBarrierResult` with entry/exit indices, gross/net returns, excursions, bars held, and reason codes

## Long and short handling

For `Long`:

- take-profit is above entry
- stop-loss is below entry

For `Short`:

- take-profit is below entry
- stop-loss is above entry

Gross return is side-aware, so short outcomes are normalized correctly.

## Same-candle ambiguous hits

When a single candle touches both take-profit and stop-loss, the rule is intentionally conservative:

- for `Long`, assume stop-loss first
- for `Short`, assume stop-loss first

This prevents optimistic intrabar bias.

## Time barrier

If no barrier is hit before the configured horizon:

- `first_hit = TimeExpired`
- `outcome = Neutral`
- exit price is the close of the expiry candle

## Cost inclusion

The evaluator records both:

- `gross_return_pct`
- `net_return_pct`

`net_return_pct` subtracts the configured fee/slippage and any entry spread supplied by candle data.

## NoData behavior

If the requested horizon extends beyond available candles:

- `outcome = NoData`
- `first_hit = NoData`
- returns remain zero

`NoData` is not treated as a win.
