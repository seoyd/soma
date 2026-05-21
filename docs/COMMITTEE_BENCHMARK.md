# Committee Benchmark

Sprint 35 adds a core-checked committee benchmark path that starts from materialized scenario rows and produces a deterministic bundle for research review.

## Flow

1. Resolve and materialize local artifacts into a `CommitteeScenarioSet`.
2. Run core-check before benchmark execution when enabled.
3. Replay committee decisions across the scenario set.
4. Build chair/risk diagnostics and decision-quality metrics.
5. Build baseline comparison, actionability, attribution, and readiness reports.
6. Write a deterministic benchmark bundle to local disk.

## Comparisons

- baseline signal summary when present
- external prediction summary when present
- always-available no-trade baseline
- risk-denied counterfactual when present

## Actionability

The benchmark counts paper approvals, reduced-size approvals, confirm-required actions, final no-trade outcomes, final denials, and readiness-eligible official actionability separately from fixture/yfinance rows.

## Attribution

Attribution is proxy-only. It reports persona vote influence, Chair decision concentration, Risk Governor concentration, and source concentration. It does **not** claim real investor reproduction.

## Readiness

Readiness stays conservative:

- fixture-only -> not ready
- yfinance-only -> research-only
- crypto-only -> not cross-market ready
- weak row-level materialization -> improve materialization first
- risk-blocked dominant -> improve Risk Governor first

No benchmark result here implies live trading readiness or profitability proof.

