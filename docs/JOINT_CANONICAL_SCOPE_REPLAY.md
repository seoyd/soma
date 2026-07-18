# Joint Canonical Scope Replay

Joint replay V2 reuses the parent V1 registration and exactly the V1 scope IDs,
digests, ranges, row order, and information cutoffs. It does not alter participant
features, labels, model parameters, training configuration, scope count, or scope
selection policy.

Momentum and Cycle/Risk receive the same verified child raw snapshot and run
independently. A technically completed no-signal Momentum result may produce a
source-bound abstention opinion; a technical failure may not. Completed Risk results
remain independent of Momentum failure. Pairing requires both completed, sealed,
source-bound opinions with the same raw rows and cutoff.

Any relationship is retrospective, advisory-only, two-round, and actionless.
Aggregation contains only relationship counts. It contains no probabilities, model
ranking, performance metrics, winner, Chair observation, vote, reward, penalty,
promotion, or execution authority.

## Sprint 59 owner-evidence result

The actual owner-evidence V2 registration verified the preserved parent
registration digest `b3c65cbced6b9ddc` and produced registration digest
`e6c593d714bbabda`. Text and JSON reports agreed, and two text replays were
byte-identical. The V2 ledger digest was `4bdfb4af6b6aba82` and the
relationship-only aggregate digest was `d3ec24be748e2cca`.

| Scope | Momentum | Momentum anchor | Cycle/Risk | Closure |
| --- | --- | --- | --- | --- |
| `joint-scope-0` | `Completed` / `BaselineStronger` / `ShadowAbstainInsufficientEvidence` | `Complete` | `Completed` / `BaselineStronger` / `ShadowAbstainInsufficientEvidence` | completed pair |
| `joint-scope-1` | `ResultClosureFailure` / `NotEvaluatedTechnicalFailure` / `ShadowAbstainTechnicalFailure` | `TechnicalFailure` | `Completed` / `UsableValidationSignal` / `ShadowPredictionResearchOnly` | technical-failure scope |

The run created three sealed source-bound opinions. The completed pair produced
one retrospective two-round actionless deliberation; it did not select a winner
or action. The full aggregate was not composed because the second scope did not
form a technically completed pair. Network counters remained zero and all Chair,
vote, and execution flags remained false.

## Sprint 60 V3 registration

V3 binds the immutable V2 registration `e6c593d714bbabda`, the exact two scope
identities, participant configuration digests, a field-level Momentum
closed-result audit policy, the closure correction policy, and the two
pre-closure result freezes. The V3 registration digest is `f3432d1552d45c70`.

The registration was issued only after the closure audit identified a stale
cross-window validator condition. It requires unchanged scope ranges and
participant configurations, unchanged pre-closure results, Scope 0
non-regression, and no result-dependent model changes. It creates no authority
and performs no provider, transport, network-consent, or credential operation.
