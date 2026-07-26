# Restart Sprint 97 Report

## Outcome

Sprint 97 implemented offline monthly/yearly candle semantic forensics,
explicit macro source policies, a qualified timeframe set, causal
revalidation, and a guarded hard-replay preregistration contract.

The persisted native responses are insufficient to prove the provider
calendar semantics behind 15 monthly and four yearly failures. All 19
failures are classified as `ProviderContractAmbiguous` and
`ExcludedUnresolved`; none is hidden, tolerance-adjusted, refetched, or
patched. Monthly and yearly model use is blocked.

Weekly remains independently qualified as
`DerivedFromCanonicalDaily`. The final source set has six qualified
timeframes and two unresolved timeframes, so full eight-timeframe fusion and
A3 are not permitted.

## Evidence summary

- monthly: 105 receipts, 17 exact, 73 within registered tolerance, 15
  unresolved;
- yearly: eight receipts, zero exact, four within registered tolerance, four
  unresolved;
- weekly: 199 registered comparisons, all exact or within registered
  tolerance;
- tolerance changes: zero;
- new network requests, transports, credential reads, and retries: zero;
- protocol events causally revalidated: 25,856;
- future, partial-candle, unqualified-view, and label reads: zero;
- qualified timeframe count: six;
- unresolved timeframe count: two;
- qualified hard-replay registration: blocked and absent;
- hard-replay execution: zero.

The prior sealed holdout `112a88137cf7db2f` remains unopened. The live second
event remains sealed, epoch three remains unregistered, and the active
committee remains three members with unchanged identity. Live outcome,
metric, parameter, normalizer, participant, governance, reward, penalty,
Chair, paper-execution, and live-execution counters remain zero.

## Model and governance boundary

The model tournament was not executed. The per-task constant benchmark
remains mandatory for any future fully qualified registration. The warning
that Raw Logistic and Interaction Logistic historical replay were worse than
constant remains active as a research constraint only.

Official Mamba-3 remains unimplemented and unevaluated. Transformer,
recurrent fusion, mixture-of-experts, attention, routing, online learning,
and trading simulation were not introduced. The Chair remains inactive.

## Implementation and verification scope

The implementation reuses the existing multi-timeframe candle rows,
normalizers, interval logic, aggregation, tolerance, as-of selection,
manual-Protobuf helpers, atomic writer, digest logic, public report boundary,
and CLI structure. Runtime evidence is ignored and uncommitted.

Focused Sprint 97 coverage contains 44 tests for evidence reopening,
metadata preservation contracts, interval semantics, completeness, fixed
tolerances, canonical root causes, source promotion/correction contracts,
qualified-set gating, causal availability, sealed holdout, zero authority,
determinism, duplicate writes, conflict/malformed rejection, and text/JSON
agreement. Existing live, historical, and multi-timeframe suites remain part
of the final sequential Default and Metal verification.

The local registration wrote 120 immutable artifacts: 113 period receipts,
two forensic aggregates, three source policies, one qualified set, and one
causal revalidation journal. It wrote no hard-replay registration. An
identical second execution wrote zero artifacts and recognized all 120 as
duplicates.

Final sequential verification passed:

- formatting and workspace checks in Default and Metal configurations;
- focused prospective, historical, Sprint 96, and Sprint 97 suites with
  54, 42, 46, and 44 passing tests respectively in each configuration;
- the complete Default workspace with 1,005 library, 404 CLI, and 12
  integration tests;
- the complete Metal workspace with 1,006 library, 404 CLI, and 12
  integration tests.
