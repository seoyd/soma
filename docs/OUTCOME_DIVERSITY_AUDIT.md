# Outcome Diversity Audit

Sprint 48 adds a deterministic audit over official outcome labels.

## Balance checks

The audit counts:

- take-profit outcomes
- stop-loss outcomes
- time-expired outcomes
- NoTrade counterfactuals
- RiskDenied counterfactuals

## Outcome entropy

Entropy summarizes how balanced the label mix is. Higher entropy means the outcome set is less concentrated in one label.

## Single-label dominance

If one label exceeds the configured maximum concentration ratio, the audit reports `SingleOutcomeDominated`.

Important constraints:

- mixed labels are necessary but not sufficient
- balanced outcomes do not imply profitable edge
- diagnostic, crypto-only, yfinance, and fixture evidence remain ineligible for official sufficiency
