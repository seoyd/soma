# Source Profile Quality Thresholds

## Purpose

Local source profiles define deterministic cadence and quality rules for
synthetic or sanitized CSV replay. These rules are conservative diagnostics,
not live market-data validation and not evidence of profitability.

## Expected Cadence

All current fixtures are minute-like, so every supported source kind uses
`FixedMillis(60_000)`.

| Source kind | Expected cadence | Gap multiplier | Allowed gaps |
| --- | --- | ---: | ---: |
| Synthetic fixture | 60 seconds | 2 | 0 |
| Korean stock CSV | 60 seconds | 2 | 0 |
| US stock CSV | 60 seconds | 2 | 0 |
| BTC crypto CSV | 60 seconds | 2 | 0 |

Korean and US profiles carry `SourceCadenceCalendarDeferred`. Weekend,
exchange-session, holiday, timezone, and real calendar validation are not
implemented. Session gaps are not implicitly allowed.

## Quality Thresholds

Each profile defines maximum timestamp gaps, duplicate timestamps, missing
optional-column ratio, volume or trade-value anomaly ratio, suspicious-scale
score, OHLC distortion count, minimum accepted rows, and minimum score.

Common defaults are zero allowed gaps, zero duplicate timestamps, zero OHLC
distortions, at least four rows, and a minimum score of `0.30`. BTC uses a
`500` anomaly ratio and `0.20` missing-optional ratio; other profiles use
`1000` and `0.25`.

Private markers and forbidden columns remain hard rejections before scoring.

## Scoring And Buckets

Scoring starts at `1.0` and subtracts deterministic penalties for excessive
gaps, missing optional coverage, volume or trade-value anomalies, suspicious
scale, and OHLC distortion.

- `Excellent`: score at least `0.95`
- `Good`: score at least `0.80`
- `Caution`: score at least `0.55`
- `Poor`: score at least the profile minimum
- `Rejected`: below minimum, too few rows, or hard parser/safety rejection

`Caution` and `Poor` describe data quality context. They do not determine
agent correctness or strategy profitability.

## Replay Policies

`RejectPoorAndBelow` is the default and blocks `Poor` and `Rejected`.
`RejectRejectedOnly` allows `Poor` with warnings.
`ReplayAllAcceptedWithWarnings` allows parsed `Caution` and `Poor` sources with
warnings. `Rejected` never enters replay under any policy.

Blocked results preserve source diagnostics, score, bucket, and reason codes
while omitting paper replay and owner sub-report results.

## Limitations

The contracts operate on local fixture timestamps only. They do not model
exchange calendars, sessions, daylight saving time, latency, live provider
quality, or real execution. Calendar-aware validation remains deferred.
