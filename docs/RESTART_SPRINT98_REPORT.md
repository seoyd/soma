# Restart Sprint 98 Report

## Delivered

Sprint 98 first reviewed and merged the preceding macro-forensics change,
then re-ran its focused and full Default/Metal baseline on the synchronized
main branch. The new work was started from that merge commit as a separate
Qualified-Six experiment.

The implementation:

- preserves the 19 unresolved month/year failures and unchanged tolerances;
- leaves the full-eight A3 registration blocked and unexecuted;
- registers one next-10-minute direction task over exactly six qualified
  views and five preregistered participants;
- derives the common eligible range, chronological partitions, additive
  sealed holdout, and minimum support from persisted evidence;
- performs fresh daily UTC walk-forward training and per-view normalization;
- persists prediction capsules before target reveal and keeps event-private
  values out of public text and JSON;
- evaluates development and validation separately;
- records paired constant and contribution comparisons without winner
  selection or governance action;
- provides status, dry-run, registration, development, and validation CLI
  modes with no holdout or network execution mode;
- adds 51 focused tests corresponding to the 51 required replay invariants.

## Executed result

Registration `e54c65c8ecbfef85` derived 25,841 common events and minimum
support 512. Development evaluated 17,395 scorable and 133 neutral events
after 560 training-only events. Validation evaluated 3,846 scorable and 30
neutral events. Invalid events were zero.

The result is mixed research evidence. Q2 had the lowest validation Brier
score, but it was worse than the constant on development. Q3 and Q4 were
worse than the constant across both partitions. No participant was selected,
promoted, added to the live roster, rewarded, penalized, or granted trading
authority.

The holdout remains closed with zero label reads, predictions, and metrics.
All live, governance, execution, network, transport, and credential counters
remain zero. Protected live artifacts and the active roster remain unchanged.

An identical completed validation invocation returned the same report digest
with zero writes, zero refits, zero predictions, and zero metric
recomputations.

## Verification

Formatting and workspace checks passed in Default and Metal configurations.
Full sequential workspace testing passed with `1,056 + 404 + 12` tests under
Default and `1,057 + 404 + 12` under Metal. The separately run focused
prospective, historical replay, multi-timeframe foundation, macro forensic,
and Qualified-Six suites passed in both configurations with 96, 43, 46, 44,
and 51 tests respectively.

For the contracts, participant definitions, aggregate results, CLI, and
authority boundary, see
[Momentum Qualified-Six Replay V1](MOMENTUM_QUALIFIED_SIX_REPLAY_V1.md).
