# Restart Sprint 15 Report

## Summary

Sprint 15 adds a four-profile local CSV source registry and connects sanitized
KR, US, BTC, and generic fixture schemas to the existing paper replay and owner
report path.

## Verification

Formatting, compilation, tests, and diff-check commands were not executed in
this implementation pass. Their result remains unknown.

## Registry

- Added local source kind, timestamp, symbol-policy, profile, registry, error,
  parse result, and quality summary types.
- Enabled synthetic fixture, Korean stock, US stock, and BTC crypto profiles.
- Rejected unknown kinds and endpoint-like profile content.

## Schemas And Fixtures

Three fake market-specific fixtures were added beside the generic fixture.
KR exercises synthetic UTC date/time normalization, US exercises
`adjusted_close` validation, and BTC exercises quote volume and trade count.

All symbols and values are explicitly fake. Source markers are synthetic.

## Validation And Normalization

The parser validates profile columns, forbidden/private markers, source
markers, timestamps, single-symbol policy, numeric fields, OHLC bounds, and
row order. Accepted data is reduced to the existing synthetic historical
dataset and `CandleSeries`.

## Replay Integration

`build_owner_learning_report_from_local_csv_source` performs:

```text
local CSV profile validation
  -> normalized synthetic historical dataset
  -> CandleSeries
  -> existing paper replay
  -> existing owner learning report
```

The report includes source kind, row quality, paper-only warning, and
not-live-ready warning. Historical episodes remain counterfactual
`NoExecution` observations.

## Tests

Test code covers all registry profiles, unknown and endpoint rejection,
market-specific fixtures, deterministic normalization, forbidden columns,
private markers, duplicate timestamps, multi-symbol rejection, owner reports,
quality summaries, fixed three-agent state, and input immutability.

Tests were not executed in this pass.

## Safety Review

No network, downloader, live provider, exchange API, Toss live API, real
broker, real order, cancellation, runtime LLM, online learning, heavy model,
UI, database, or eight-agent activation was added.

## Deferred

Real sanitized imports, timezone-aware market calendars, multi-symbol files,
quoted general-purpose CSV, corporate-action policy, durable storage, UI, and
live execution remain deferred.

## Next Sprint

Run the accumulated verification gate before adding any import or persistence
surface.
