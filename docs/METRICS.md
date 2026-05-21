# Metrics

Sprint 06 adds deterministic, cost-aware evaluation metrics.

## Trade metrics

`TradeMetrics` reports:

- trade counts
- win / loss / neutral counts
- win rate
- average win / loss
- gross return
- net return
- profit factor
- max drawdown
- average bars held

## Decision metrics

`DecisionMetrics` reports:

- total decisions
- executed decisions
- denied-by-risk decisions
- no-trade decisions
- require-confirm count
- approve-candidate count
- reason code counts

## NoTrade metrics

`NoTradeMetrics` captures defensive silence:

- no-trade count
- avoided-loss count
- missed-gain count
- average avoided-loss score
- average missed-gain penalty
- net silence value

## Risk Governor metrics

`RiskGovernorMetrics` captures veto behavior:

- denied count
- emergency-stop count
- cooldown count
- avoided-loss count
- missed-gain count
- defensive value
- opportunity cost

## Calibration metrics

`CalibrationMetrics` provides:

- Brier score for `p_win`
- fixed probability bins
- predicted average per bin
- actual win rate per bin
- simple expected calibration error

## Regime metrics

`RegimeMetrics` groups trade, decision, no-trade, and risk metrics by regime so `Unknown`, `Panic`, and `HighVolatility` remain visible separately.

## Fold / aggregate reports

`FoldReport` stores per-fold:

- fold boundaries
- row counts
- leakage report
- fold metrics
- regime split metrics
- persona summary
- Chair summary

`WalkForwardReport` stores:

- config
- fold reports
- aggregate metrics
- feature schema
- report-level reason codes
