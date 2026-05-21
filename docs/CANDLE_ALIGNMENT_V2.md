# Candle Alignment V2

Candle alignment is deterministic and local-only.

## Matching rules
- symbol matching is exact after conservative normalization
- timestamp matching is exact by default
- tolerance matching is explicit and bounded by config
- horizon matching is strict by default

## Explicit statuses
- `MatchedExact`
- `MatchedWithTolerance`
- `MissingCandleSeries`
- `MissingTimestamp`
- `WrongSymbol`
- `WrongHorizon`
- `GapDetected`
- `DuplicateTimestamp`
- `InsufficientFutureBars`
- `BadDataQuality`
- `RejectedNoLookahead`

## Safety
- no hidden symbol mapping
- no hidden timestamp repair
- future windows must exist locally
- matches that would consume future candles before the scenario timestamp are rejected as no-lookahead unsafe
