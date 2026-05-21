# Sprint 92 KRX warning closure

Sprint 92 follows Sprint 91 because Sprint 91 stopped at `KrxEvidenceReducedWithWarnings`. The remaining warning was the isolated raw-archive redaction sentinel, so Sprint 92 focuses on explicit closure instead of starting a new family reduction.

## Why `KrxEvidence` stays primary

- Sprint 91 left `KrxEvidence` as the queue primary.
- The raw archive secret-safety sentinel remains separate.
- The real workspace gates still time out around 300 seconds.
- `DashboardRenderer` may only be considered through an entry gate, not through reduction work.

## Sprint 92 outcome

- warning closure: `KrxWarningsClosedWithIsolatedSentinel`
- genuine reduction gate: `KrxEvidenceReducedWithIsolatedSentinel`
- DashboardRenderer entry gate: `DashboardRendererEntryBlockedByUnknownGateCause`
- queue: still conservative with `KrxEvidence` primary because the 300-second no-run/full timeouts remain unattributed

## No-run/full distinction

A no-run timeout is only compile-stage evidence. It does not imply a full workspace pass. A full workspace timeout is still a blocked ship gate until a finished passing run is observed.

## No fake pass

Sprint 92 does not claim:

- full workspace acceptance
- live trading readiness
- broker/order/account readiness
- DashboardRenderer reduction completion
