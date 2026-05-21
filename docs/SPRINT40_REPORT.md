# Sprint 40 Report

Sprint 40 introduces a deterministic, research-only core performance reporting stack.

## Added runtime pieces
- core artifact inventory
- signal quality report
- committee value attribution report
- risk-governor value report
- no-trade value report
- latency and budget report
- regression guard
- bottleneck report
- scorecard runner and bundle writer

## CLI
- `soma-experiment core-performance --config <...>`
- `soma-experiment core-bottleneck --config <...>`
- `soma-experiment core-regression --config <...>`

All commands reject remote paths and remain paper-only.
