# KRX Candle Sufficiency

Sprint 50 adds a KRX candle sufficiency report that explains whether local KRX candle series are ready for no-lookahead outcome linkage.

## What is checked

- official-readiness gates from canonical validation
- symbol and timeframe alignment
- timestamp ordering and gap pressure
- required future-window depth from the barrier-profile registry
- no-lookahead safety

## Status interpretation

- `HealthyKRXCandles`: official-ready series with enough future bars
- `MissingOfficialCandles`: no official-ready series yet
- `MissingFutureWindows`: candles exist but the forward window is too short
- `MissingPreflight` / `MissingProvenance`: official readiness is blocked upstream
- `TimestampAlignmentWeak`, `DataQualityTooLow`, `InsufficientRows`: local data still needs repair or extension

Missing future windows block outcome links even when canonical CSVs already exist.
