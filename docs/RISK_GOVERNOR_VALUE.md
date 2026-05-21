# Risk Governor Value

Sprint 40 separates defensive value from overblocking.

## What is measured
- approvals, reductions, no-trades, denials
- hard vetoes vs soft-threshold denials
- avoided loss vs missed gain
- denial-rate behavior under weak evidence

## Interpretation
Valid denials are not bugs. Repeated soft denials that block more gain than avoided loss are flagged for review, but hard veto protections remain intact.
