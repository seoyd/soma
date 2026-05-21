# Report Diff

Campaign diff compares current evidence against a previous campaign report.

## What it checks

- passed runs
- usable dataset coverage
- outcome count
- average net return
- worst drawdown
- average calibration brier
- data quality score
- risk defensive value
- denial / no-trade rate
- persona redundancy warnings

## Conservative interpretation

Net return alone is not enough.

- worse drawdown is a regression even if return improves
- worse calibration is a regression
- worse data quality is a regression
- more usable data counts as improvement only when it is not paired with safety degradation

If there is no previous report, the diff stays unavailable instead of pretending to compare.
