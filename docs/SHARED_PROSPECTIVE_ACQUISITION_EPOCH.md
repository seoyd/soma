# Shared Prospective Acquisition Epoch

One future finalized BTC row may be requested by a later explicitly authorized epoch at most once and fan out only as immutable raw evidence. Each registered challenge independently checks its sealed capsule, provider, series, cutoff, finality, duplication, and vault transition.

Feature vectors, representations, probabilities, support decisions, labels, journals, and verdicts remain challenge-specific. The epoch has one request, concurrency one, zero retries, no poller, and no background task. An unclassified prior receipt leaves the epoch sealed and network-ineligible; it never erases the old Momentum request history.
# Shared prospective acquisition epoch

## Sprint 52 qualification boundary

A shared epoch is distinct from the prior Momentum attempt.  It carries the
prior receipt digest and computed eligibility, registers only Momentum and
Cycle/Risk in deterministic order, and permits at most one request with no
retry.  Qualification is blocked whenever the receipt cannot establish a known
root cause; sealing never grants network permission by itself.

Only a canonical finalized raw-row reference may cross the challenge boundary.
Feature vectors, model representations, probabilities, support decisions,
labels, journals, and verdicts remain challenge-specific.
