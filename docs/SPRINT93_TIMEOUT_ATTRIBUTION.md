# Sprint 93 timeout attribution

Sprint 93 follows Sprint 92 because Sprint 92 closed the KRX warning state conservatively but left the real workspace timeout unattributed. The next safe step is timeout attribution, not DashboardRenderer reduction.

## Why KRX still controls the queue until attribution finishes

- Sprint 92 ended with `KrxWarningsClosedWithIsolatedSentinel`, not with a proven non-KRX timeout cause.
- `DashboardRendererEntryBlockedByUnknownGateCause` remained the previous authoritative entry state.
- Queue movement only becomes safe after explicit KRX non-primary proof or an equally conservative decision.

## What Sprint 93 adds

Sprint 93 adds a local deterministic timeout-attribution bundle that keeps diagnostic observation separate from the quiet acceptance gates:

- `RealWorkspaceTimeoutAttributionReport`
- diagnostic no-run/full pass reports
- cargo message capture, rustc snapshot, and target-dir growth reports
- cargo target progress timeline
- KRX non-primary proof and primary-blocker report
- unknown timeout closure and attribution decision
- DashboardRenderer entry release gate/report and reduction hold report
- queue v9, recovery v10, safety coverage v9, and the read-only Control Tower panel

## Diagnostic commands vs quiet gates

- `real-no-run-diagnostic-pass` and `real-full-diagnostic-pass` improve visibility only.
- They do **not** replace `cargo test --workspace --no-run --quiet` or `cargo test --workspace --quiet`.
- A diagnostic pass must never be reported as final gate acceptance.

## Why DashboardRenderer does not start yet

Even when Sprint 93 can release DashboardRenderer **entry**, Sprint 93 still keeps DashboardRenderer **reduction** held. Entry release only changes next-family eligibility; it does not start implementation reduction work.

## No fake pass, no fake timing

Sprint 93 stays local-only and deterministic, but it does not fabricate elapsed time, pass/fail, or non-primary proof. If evidence is incomplete, the queue remains blocked. Diagnostic logs remain secret-free, the isolated raw-archive sentinel stays preserved, and KIS/KRX remain market-data/reference-only.
