# KRX Evidence Real Reduction

`KrxEvidenceRealReductionConfig` keeps Sprint 91 local-only and fixes the target family to `KrxEvidence`.

The reduction reports separate:

1. assertion migration into `tests/krx_evidence_suite.rs`,
2. fixture/setup reduction through shared harness reuse,
3. missing-auth preservation,
4. endpoint-template preservation,
5. source-boundary preservation.

Historical donor filenames remain explicit even when a donor file is part of historical lineage only.
