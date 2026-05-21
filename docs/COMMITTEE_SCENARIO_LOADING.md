# Committee scenario loading

Sprint 33 adds `CommitteeScenarioLoader` to turn local report summaries into deterministic committee rows.

## Supported inputs

- fixture-only committee scenarios
- evidence-lane plan summaries
- source-aware benchmark summaries
- yfinance research summaries
- official/core-checked benchmark summaries

## Summary-derived rows

Some upstream reports are lightweight summaries, not row-level datasets. In those cases the loader creates **bounded summary-derived rows** and marks them with `SummaryDerived`.

## Source boundaries

- fixture rows stay fixture-only
- synthetic test rows stay synthetic-only
- yfinance rows stay research-only
- official rows need explicit provenance summaries when available

The loader is local-path-only and rejects remote paths.

