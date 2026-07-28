# Momentum T10 Actionability Design V1

## Purpose

This design replaces neither the failed Sprint 102 direction task nor its
receipts. It uses consumed development and validation evidence to register a
volatility-normalized, three-state T10 research label and an unexecuted
two-stage selective architecture.

The untouched holdout has already been split without reading labels. Fresh
challenger validation and final holdout remain closed.

## Candidate labels

The scale reuses the existing population standard deviation of the preceding
144 simple returns from 145 already-closed ten-minute candles. A registered
positive finite floor is used only when that past-only volatility is zero.
Events without the complete registered past-only context are ineligible for
candidate-label derivation; they are not assigned the zero-volatility floor.
The evidence reader reopens only the canonical one-minute source, its verified
ten-minute aggregate, and sealed protocol metadata; daily, weekly, monthly, and
yearly views are not loaded.

Exactly three multipliers are registered: `0.25`, `0.50`, and `1.00`.

- `ActionableUp`: future return is greater than `k × sigma`.
- `Abstain`: absolute future return is less than or equal to `k × sigma`.
- `ActionableDown`: future return is less than `-k × sigma`.

Equality belongs to `Abstain`. These labels are volatility-normalized research
labels, not fee-, spread-, slippage-, P&L-, or profitability-aware labels.

The frozen selector requires adequate support and stable Up, Down, and Abstain
prevalence in both consumed partitions. It chooses the largest passing
multiplier. No passing multiplier blocks all selective participant and pair
registration.

Even when a multiplier passes, the architecture registration is blocked if the
timestamp-derived fresh-validation child is smaller than the frozen minimum
support requirement. This is recorded as
`FreshValidationInsufficientSupport`; the 50/50 split is never adjusted to
manufacture support.

## Selective architecture

When a threshold passes, the registration contains:

- O0 opportunity prevalence constant;
- O1 frozen ten-minute anchor opportunity logistic;
- O2 frozen 69-dimensional compact opportunity logistic at exactly four times
  standard L2;
- D0 actionable-direction prevalence constant;
- D1 frozen ten-minute anchor direction logistic;
- D2 frozen 69-dimensional compact direction logistic at exactly four times
  standard L2;
- S0 = O0 + D0, S1 = O1 + D1, and S2 = O2 + D2.

Direction participants exclude `Abstain` from training. Cross-pairs,
calibration, ensembles, interaction expansion, routing, and sequence models are
not registered.

The future policy is chronological daily UTC prequential refitting with
training-only normalizers and previously revealed labels. Separate opportunity
and direction Brier gates are mandatory. Correctness and coverage cannot
override either Brier failure.

The registration separately freezes opportunity-head, direction-head, and
end-to-end selective metrics. The selective metrics cover abstention, coverage,
opportunity precision/recall, direction quality on actionable evidence, false
and missed actions, finite values, chronology, and leakage. They contain no P&L,
fee, spread, slippage, Sharpe, drawdown, or position-sizing metric.

Inference is also frozen: opportunity probability below `0.5` abstains;
otherwise direction probability at or above `0.5` maps to Up and a lower value
maps to Down.

## Authority boundary

Threshold selection uses consumed evidence only. The architecture is registered
but not trained. No opportunity or direction fit, selective prediction,
selective evaluation, fresh-validation read, final-holdout read, T30/T60
execution, macro-view load, live operation, governance action, network request,
or trading action is authorized.
