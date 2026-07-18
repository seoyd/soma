# Momentum Closed-Result Contract

## Scope

The closed-result contract converts an immutable Momentum campaign result and its
immutable chronological regime reference into a closed report. It is a closure
and validation boundary only: it does not alter scopes, snapshots, features,
labels, candidates, encoder state, training settings, support policy, or model
output.

## V3 field audit

The V3 audit reconstructs the closure semantics and compares the regime identity,
row and campaign counts, execution and diagnostic states, verdict mapping,
operational result, window counts, support evidence, diagnostics, reason codes,
execution trace, and report digest. It separately preserves the shared
validator's typed sanitized error. The audit is deterministic and uses only
redacted digests, enum names, and counts; it emits no market rows, probabilities,
model parameters, paths, or backtraces.

The pre-closure freeze binds the campaign report digest, window count, final
verdict, no-signal and selected-checkpoint counts, support-count vector, encoder
digest, pack digest, and derived-snapshot digest. Its digest is included in the
V3 registration and checked again before replay.

## Proven stale condition

The pre-repair audit passed every closure-builder field comparison for the target
scope, but the shared validator returned `MissingRequiredMetric`. The first
contract failure was the legacy no-signal/checkpoint exclusivity rule: a campaign
with at least one no-signal window and at least one independently selected
checkpoint window was rejected even though those counts are per-window aggregate
facts and may legitimately coexist.

The expected semantic value was `mixed_window_counts_permitted`; the legacy
validator's actual semantic result was `rejected_by_legacy_validator`. This is a
`StaleValidatorContract`, not a model, scope, snapshot, builder-field, or regime
reference defect.

## Minimal repair

The repair removes only that cross-window exclusivity predicate from the shared
closed-result validator. It retains checkpoint bounds, support-count bounds,
accepted-version bounds, exact execution-trace length, and non-empty report
digest validation. The joint wrapper now preserves the actual typed closure error
as a sanitized reason code if any remaining validator check fails.

No model output, campaign input, threshold, reason code, raw scope, or snapshot
is changed by this repair.

## Immutable freeze evidence

| Scope | Pre-closure digest | Campaign windows | No-signal | Selected checkpoint | Final verdict |
| --- | --- | ---: | ---: | ---: | --- |
| `joint-scope-0` | `2e640a195c0b4177` | 2 | 0 | 2 | `InSupportUsableSignalButLinearStrongerOnThisSeries` |
| `joint-scope-1` | `bc840c1a4f5cfe00` | 2 | 1 | 1 | `TemporalOutOfSupportAbstention` |

The two pre-closure digests were unchanged after the closure repair. Scope 0's
field audit remained valid; Scope 1's repaired field audit is valid because the
previously stale cross-window condition no longer rejects an internally coherent
closed result.

## V3 registration

V3 binds parent V2 registration `e6c593d714bbabda`, the two exact scope IDs and
digests, Momentum and Cycle/Risk configuration digests, the closure-audit policy,
the `StaleValidatorContract` correction classification, the corrected closure
policy, and both pre-closure freeze digests. Its registration digest is
`f3432d1552d45c70`.

The registration requires unchanged scope ranges and participant configuration,
unchanged pre-closure results, Scope 0 non-regression, and no result-dependent
model change. It remains offline and advisory-only: all provider, transport,
network-consent, and credential counters are zero; no Chair, vote, reward,
penalty, promotion, or execution authority is created.
