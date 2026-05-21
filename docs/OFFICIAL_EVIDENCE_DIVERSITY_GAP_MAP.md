# Official Evidence Diversity Gap Map

Sprint 48 adds a bounded gap map for official non-crypto evidence diversity.

## What it measures

The report tracks current versus target coverage for:

- official complete rows
- symbol diversity
- timeframe diversity
- horizon diversity
- outcome-label diversity (`TakeProfit`, `StopLoss`, `TimeExpired`)
- NoTrade and RiskDenied counterfactual depth

## Sprint 47 limitation

Sprint 47 could prove plumbing on one or two official rows, but that was still too shallow for research usefulness. Two rows with only take-profit outcomes can pass plumbing while remaining conservative and insufficient for committee research readiness.

## Gap routing

The gap map routes the dominant blocker into explicit statuses such as:

- `SingleOutcomeDominated`
- `NeedMoreSymbols`
- `NeedMoreTimeframes`
- `NeedMoreHorizons`
- `NeedStopLossOutcomes`
- `NeedTimeExpiredOutcomes`
- `NeedCounterfactualDepth`
- `DiagnosticOnly`

The report is intentionally conservative: mixed labels help, but they never imply profitability or deployment readiness.
