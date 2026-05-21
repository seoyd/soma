# Timeframe and Timestamp Alignment

Sprint 42 splits candle matching into timeframe alignment and timestamp alignment v2.

## Timeframe
- exact match is preferred
- aggregation is explicit and disabled by default
- downsampled coverage is diagnostic-only
- upsample does not create information and is rejected
- missing timeframe metadata blocks official-ready use

## Timestamp
- exact match is preferred
- tolerance matching is explicit and bounded
- session-daily matching is explicit and reason-coded
- duplicate timestamps and gaps are reported, not repaired silently
- insufficient future windows block benchmark-ready use

## No-lookahead
- matching must not consume future information before the scenario timestamp
- unsafe matches are rejected
- local candles are required for replayability and auditability
