# BTC support-gate closure

The BTC cross-regime diagnostic distinguishes four facts that were previously
collapsed into one unavailable result: support-envelope construction, gate
applicability, validation support decision, and the operational outcome.

An envelope is constructed from the selected candidate's train-only frozen
representations. Its trace records the deterministic digest, sample and
dimension counts, finite-statistic evidence, and constant-dimension count.
Construction is not inferred from a threshold decision.

Applicability is then determined independently from finite representation
metrics, shape, and minimum sample count. An applicable gate may legitimately
return `OutOfSupport`; a threshold breach is a research rejection, not a gate
failure. Required metrics are emitted in a stable order: validation sample
count, validation coverage, mean and maximum standardized shift, mean log
variance ratio, and out-of-support fraction. The trace includes the first
breach, measured value, configured threshold, and required flag. No optional
metric is invented or silently substituted.

When validation is not `InSupport`, the test support decision is
`NotEvaluated`. This states that the support gate did not authorize test
qualification; it is neither `SupportGateUnavailable` nor a fabricated test
rejection. Existing research-only counterfactual calculations remain visible
as diagnostics and never select, promote, vote, or execute a version.

The diagnostic also runs a label-free, train-history-only applicability audit.
It uses deterministic chronological prefix-to-suffix folds over the same
selected frozen representations. The audit has no test rows and changes no
support threshold, candidate, label, or selection rule. Its result is an
applicability warning only; it cannot make a model eligible.

The support aggregate is separate from the representation aggregate. It counts
validation InSupport, validation OutOfSupport, insufficient evidence, true
gate-unavailable paths, test InSupport, test OutOfSupport, and accepted
research-only predictive versions. `SparseSupportQualifiedEvidence` requires a
real support-qualified version; a zero-qualified result is instead
`NoSupportQualifiedEvidence`. Representation stability is reported alongside,
not used as a substitute for support qualification.

All diagnostic execution is offline over frozen local packs. Provider calls,
transport construction, network-consent reads, and credential reads are zero.
The CLI output is deterministic text or JSON and preserves the existing
ShadowOnly boundary.
