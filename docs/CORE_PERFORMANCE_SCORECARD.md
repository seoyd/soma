# Core Performance Scorecard

Sprint 40 adds a deterministic, research-only scorecard that aggregates artifact inventory, signal quality, committee value attribution, risk-governor value, no-trade value, latency budget, regression checks, and a bottleneck recommendation.

## Guarantees
- local paths only
- no live trading or broker/account execution
- no runtime LLM or Mamba runtime
- same inputs produce the same scorecard bundle

## Final status intent
`CorePerformanceHealthyForResearch` is still paper-only and never a profitability or live-readiness claim. Controlled, fixture, crypto-only, and yfinance-only evidence stay explicitly non-official.

## Output bundle
`target/soma_core_performance/<scorecard_id>/`
- `artifact_inventory.txt`
- `signal_quality.txt`
- `committee_value_attribution.txt`
- `risk_governor_value.txt`
- `no_trade_value.txt`
- `latency_budget.txt`
- `regression_guard.txt`
- `bottleneck_report.txt`
- `core_performance_scorecard.txt`
- `core_performance_scorecard.json`
