# Joint Momentum Failure Forensics

Joint Momentum replay V2 records execution health separately from model-evidence
outcome and anchor-audit availability. It is offline-only and never initializes a
provider, transport, credential, prospective-state, Chair, vote, reward, penalty,
promotion, or execution path.

Each scope/participant trace records deterministic stage status, a sanitized error
code when applicable, reason codes, and an artifact digest. The trace does not
contain OHLCV values, paths, credentials, model parameters, probabilities, or trade
actions. A technical failure maps to `NotEvaluatedTechnicalFailure` and
`ShadowAbstainTechnicalFailure`; it is not represented as a no-signal result.

The legacy V1 replay remains a historical record. V2 reproduces it for forensic
comparison without changing its committed status, then uses the first failed V2
stage to classify the root cause before a corrected replay is considered.

## Sprint 59 owner-evidence materialization

The owner-local immutable replay ran twice in text form with byte-identical
sanitized output, and the JSON report agreed on every reported scope, stage,
root-cause, count, and digest field. Both registered scopes reproduced the legacy
classification below; no provider, transport, credential, or authority path ran.

| Scope | First failed stage | Root cause | Trace digest | Forensic digest |
| --- | --- | --- | --- | --- |
| `joint-scope-0` | `DerivedSnapshotIdentity` | `DerivedSnapshotIdentityMismatch` | `8cae91d151205036` | `00eb6bb55c61a6d5` |
| `joint-scope-1` | `DerivedSnapshotIdentity` | `DerivedSnapshotIdentityMismatch` | `e242ac69277b9379` | `c9c94273acff43b5` |

This is a legacy-adapter identity classification, not a claim that the V2 child
snapshot is invalid. The corrected V2 replay is recorded separately.
