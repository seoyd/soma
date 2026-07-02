# Owner Report Regression

## Inputs

`OwnerLearningReport` accepts a completed immutable `PaperReplayResult`.
Historical integration first produces the same replay result through the
existing three-agent paper path. No report function reads a broker, account,
network response, or mutable global state.

## Outputs

The report exposes:

- overall paper episode, trade, NoTrade, and Risk denial counts,
- per-agent version, voice, tier, status, cooldown, memory, reward, and penalty,
- Chair reward, penalty, cooldown, quarantine, and candidate counts,
- Risk Governor denial categories,
- sandbox isolation status,
- owner advisory rejection counts and stable explanations.

The historical report identifies its source as fixture or synthetic.

## Renderers

Text, Markdown, and JSON-like renderers use stable agent ordering, stable reason
ordering, and fixed numeric formatting. They return strings and perform no
file IO. Unsafe credential, token, account, raw provider, private mapping,
local-private, environment-file, and temporary-instruction markers cause
rejection or line-level redaction.

## Review Console

The function-level console supports summary, agent, Risk, sandbox, owner
advisory, and reason-code explanation queries. Responses explicitly disable
state mutation, execution, promotion, and cooldown clearing. There are no
approve, order, cancel, mutate, or network commands.

## Regression Boundary

Report construction and every command borrow their inputs immutably. They
cannot change replay states, version journals, sandbox candidates, owner
reviews, or Risk Governor decisions.

This layer is paper-only, read-only, and deterministic. It is not live trading,
not a live UI, and not proof of production readiness.

## Known Limitations

- Attribution remains role-based.
- Output redaction is conservative marker matching.
- Reports have no persistence or access-control layer.
- Historical adapter episodes are counterfactual NoExecution observations,
  not claimed fills or realized live performance.
