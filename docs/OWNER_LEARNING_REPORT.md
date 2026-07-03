# Owner Learning Report

## Purpose

`build_owner_learning_report` converts one completed `PaperReplayResult` into a
deterministic owner-readable view. It reads replay summaries and audit results
only. It does not call Chair, Risk Governor, `PaperBroker`, a network, or an
LLM.

The report begins with four fixed statements:

- Paper-only report.
- Not live trading ready.
- Risk Governor remains final veto.
- Owner input is advisory only.

## Report Sections

The report contains:

1. safety status,
2. overall paper replay counts,
3. three agent learning views,
4. Chair reward and penalty counts,
5. Risk Governor denial counts,
6. sandbox candidate safety,
7. owner advisory outcomes,
8. a deferred/live-readiness warning.

Each agent view shows start and end versions, voice and delta, tier, status,
cooldown, wins, losses, avoided losses, missed gains, high-confidence misses,
doctrine violations, total reward, total penalty, net reward/penalty, sandbox
candidate count, stable explanation, and reason codes.

## Read-Only Boundary

The builder accepts `&PaperReplayResult`. Renderers accept
`&OwnerLearningReport`. They cannot mutate canonical state, clear cooldown,
promote a sandbox candidate, approve a trade, submit an order, or alter Risk
Governor configuration.

No current report function performs file IO. Text, Markdown, and JSON-like
renderers return strings to their caller.

## Owner Review Commands

`handle_owner_review_command` supports:

- `ShowSummary`,
- `ShowAgent`,
- `ShowRisk`,
- `ShowSandbox`,
- `ShowOwnerAdvisory`,
- `ExplainReasonCodes`.

Every response explicitly reports no state mutation and disables order
execution, sandbox promotion, and cooldown clearing. This is a pure
function-level console, not an interactive terminal or web UI.

## Explanation And Secret Safety

Owner rejection text reuses stable reason-code templates. No runtime LLM or
generated prose service is involved.

Renderer output removes complete lines containing credential, authorization,
token, account, raw provider response, private mapping, local private content,
environment-file, or temporary-instruction markers. Redaction is explicit in
the returned text. Report identifiers containing such material are rejected.

## Boundaries

This report is paper-only and read-only. It is not evidence of live trading
readiness. It adds no real broker, order placement, cancellation, private
account access, live network, online learning, heavy model, or eight-agent
activation path.

A future UI may display the same immutable schema, but UI, database
persistence, authentication, and remote export remain deferred.

## Historical Fixture Reports

`build_owner_learning_report_from_historical_replay` first converts a validated
synthetic dataset into the existing paper replay path and then calls the same
report builder. The report records the fixture source in
`generated_from_replay_id`. It does not label counterfactual observations as
executed trades.

Local source reports additionally attach `LocalDataQualitySummary`. Text,
Markdown, and JSON-like output can show source kind, accepted and rejected row
counts, timestamp range, monotonicity, trade-value availability, and close
range. This metadata does not change replay state.

## Batch Reports

`build_batch_owner_learning_report` combines immutable source and agent
performance tables from `BatchReplayResult`. It references the per-source
owner reports already produced by accepted replays and does not rerun or
mutate replay state.

`render_batch_owner_learning_report_text` emits fixed paper-only and
not-live-ready warnings, source and agent summaries, Risk Governor and sandbox
sections, rejected sources, and deferred items. It reuses the same line-level
private-material redaction as the single-replay renderers.

The batch report also records replay mode, source order policy, actual
processing order, cross-source quality diagnostics, deduplicated source
warnings, and the three-agent cross-source consistency table. Fixed report
statements identify synthetic/sanitized local data, make no profitability
claim, retain Risk Governor veto, and keep owner input advisory.

Batch values describe synthetic paper replay only. They are not claims of
profitability, production data quality, or live execution readiness.
