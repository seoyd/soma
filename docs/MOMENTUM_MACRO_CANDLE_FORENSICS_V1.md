# Momentum Macro Candle Forensics V1

## Purpose and authority

This lane qualifies weekly, monthly, and yearly historical candle semantics
without network access. It is historical research only. It cannot train a
model, open the sealed holdout, read a live outcome, change the committee,
apply a reward or penalty, invoke the Chair, or execute a paper or live trade.

The registered absolute `1e-12` and relative `1e-10` tolerances remain fixed
and apply only to accumulated volume and trade value. OHLC equality remains
exact. No per-period exception, selective summation order, or tolerance
enlargement is available.

## Persisted-evidence boundary

The native page receipts preserve normalized candle intervals and values,
the exclusive request boundary, and the source-response digest. The
historical receipts do not preserve the provider's KST timestamp,
`first_day_of_period`, or last-trade timestamp.

Those missing fields cannot be reconstructed from a response digest. The
implementation therefore does not infer provider calendar semantics, replay
a request, construct a transport, or read credentials. A failed value
comparison whose cause depends on the absent metadata is classified as
`ProviderContractAmbiguous`, disposed as `ExcludedUnresolved`, and reported
under `InsufficientPersistedNativeEvidence`.

## Per-period classification

One manual-Protobuf forensic receipt is created for each compared monthly and
yearly period. Each receipt binds:

- native and derived candle identities;
- native and derived open/close-exclusive intervals;
- the exclusive request boundary and response identity;
- native normalized-row and derived daily-source identities;
- boundary, completeness, and value classifications;
- exactly one supported root cause for a failed comparison;
- the canonical disposition and semantic receipt digest.

Raw native and derived OHLCV values and per-period differences remain private
inside ignored runtime evidence.

Boundary classification compares intervals rather than file order. Current
unfinished periods are unavailable until semantic close. A source boundary
inside a calendar period, a missing daily interval, and a no-trade interval
are distinct completeness states. A boundary mismatch is decided before
numeric comparison.

## Verified result

| Timeframe | Compared | Exact | Registered tolerance | Failed | Root cause for failures | Policy |
|---|---:|---:|---:|---:|---|---|
| `1w` | 199 | 74 | 125 | 0 | none | `DerivedFromCanonicalDaily` |
| `1mo` | 105 | 17 | 73 | 15 | `ProviderContractAmbiguous` | `ExcludedUnresolved` |
| `1y` | 8 | 0 | 4 | 4 | `ProviderContractAmbiguous` | `ExcludedUnresolved` |

Weekly qualification is independent of the month/year result. Its registered
native samples remain exact or within the unchanged accumulation tolerance,
and every normalized weekly interval agrees with the registered weekly
calendar boundary.

The 15 monthly and four yearly failures are neither hidden nor averaged.
Because their source metadata is incomplete, neither native promotion nor a
derived-aggregation correction is justified. The old derived indexes remain
immutable audit evidence, and no replacement index is fabricated.

## Qualified source set and causality

The qualified set is:

```text
qualified: 1m, 3m, 5m, 10m, 1d, 1w
excluded unresolved: 1mo, 1y
qualified count: 6
unresolved count: 2
full eight-timeframe replay allowed: false
```

The causal revalidation reopens all 25,856 protocol events. It selects only
the six qualified sources through `close <= prediction timestamp`, records
the two unqualified macro views as blocked, and never reads them as model
inputs. Future, partial-candle, unqualified-view, and label access counts are
zero.

The sealed holdout identity remains `112a88137cf7db2f`. Labels, metrics,
per-event predictions, and aggregate comparison remain closed. No new
holdout boundary is needed because the qualified-source result does not
authorize a replay.

## Hard replay rule

A qualified hard-replay registration can be constructed only from an
all-eight qualified set. Its three tasks have distinct timestamp, horizon,
label, cadence, eligible-range, and minimum-context policies. It freezes A0
through A5, makes the per-task constant benchmark mandatory, limits research
models to logistic baselines and block/fusion logistic models, and preserves
the warning that prior historical logistic replays were worse than constant.

The present six-of-eight set cannot register A3 full fusion. No reduced or
full model replay, training, tournament, ablation execution, or metric
computation occurs in this sprint.

## CLI and persistence

```text
--momentum-mtf-macro-forensics --status --output-format text|json
--momentum-mtf-macro-forensics --dry-run --output-format text|json
--momentum-mtf-macro-forensics --execute-local --output-format json
--momentum-mtf-hard-replay-registration --status --output-format text|json
```

All modes are offline-only. The CLI rejects network, live outcome/opening,
training, holdout, reward, Chair, and trading authority flags.

Every artifact uses the existing manual-Protobuf and atomic persistence
path: create-new temporary write, flush, `sync_all`, reopen and decode,
semantic-digest verification, atomic rename, and final reopen verification.
Identical reruns perform zero new writes; malformed and conflicting artifacts
are rejected.

The verified local execution persisted 113 forensic receipts, two
aggregates, three policies, one qualified set, and one causal revalidation
journal. No hard-replay registration was persisted. The identical rerun
performed zero writes and reopened all 120 existing artifacts as semantic
duplicates.

## Verification

Default and Metal formatting/check gates passed. The 44 focused forensic
tests passed in both configurations, together with the related prospective,
historical, and prior multi-timeframe suites. Full sequential workspace
testing passed with 1,005 + 404 + 12 tests under Default and
1,006 + 404 + 12 tests under Metal.
