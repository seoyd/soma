# BTC cross-regime diagnostic closure

BTC cross-regime replay is an offline, deterministic, ShadowOnly research
operation over immutable accepted snapshots and independently frozen regime
packs. It performs no provider request, transport construction, credential
read, or network-consent read.

Each sealed regime report keeps four separate layers: execution health,
diagnostic completeness, model-evidence outcome, and operational Shadow result.
This prevents a negative research outcome from being mistaken for an execution
error.

No usable validation signal is a completed abstention: the checkpoint and test
paths remain inapplicable, the test stays sealed, no predictive version is
accepted, and the cross-regime result may be `PredominantlyNoUsableSignal`.
A support gate that cannot be evaluated is likewise a completed, explicit
Shadow abstention. Zero accepted predictive versions is a safety and evidence
result, not a runtime failure.

`DiagnosticFailure` is reserved for an untrustworthy technical result, such as
a pack or config-digest failure, a missing required metric, a failed report
invariant, or a nondeterministic replay. Optional unavailable metrics do not
qualify. Aggregation consumes only sealed reports after their trace and report
digests have been verified.

The offline CLI is `--btc-cross-regime-diagnostics` with the existing local
historical campaign configuration. Text and JSON output are sanitized and
deterministic. A valid negative result exits successfully; a genuine technical
diagnostic failure exits nonzero.

All outputs remain research-only Shadow outputs. They cannot vote, promote,
execute, alter the Chair or Risk Governor, or establish live-trading readiness.
