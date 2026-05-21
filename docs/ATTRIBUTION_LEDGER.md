# Attribution Ledger

## Ledger records

Sprint 04 adds a minimal attribution ledger around each simulated decision.

### DecisionRecord

`DecisionRecord` captures the decision-time state:

- decision id
- timestamp
- symbol
- `SignalOutput`
- investor votes
- `ChairOutput`
- `RiskDecision`
- optional `TradeProposal`
- execution selection flag
- optional paper order id
- reason codes
- audit event id

### OutcomeRecord

`OutcomeRecord` captures what happened after the decision:

- executed / denied / no-trade flags
- optional executed `TripleBarrierResult`
- optional hypothetical result
- realized net return
- avoided-loss score
- missed-gain penalty
- attribution records
- shadow outcome records
- reason codes

## AttributionRecord

Each persona gets a role-scoped attribution entry:

- `persona_id`
- `selected_for_decision`
- `stance`
- `conviction`
- `voice_power`
- `contribution_score`
- `counterfactual_role`
- reason codes

Current roles are:

- `SupportedFinalDecision`
- `OpposedFinalDecision`
- `ForcedContrarian`
- `ShadowOnly`
- `RiskVetoAligned`
- `RiskVetoOpposed`

The model is intentionally simple and deterministic; Sprint 04 does **not** implement heavy Shapley-style attribution.

## ShadowOutcomeRecord

Sprint 03 introduced shadow vote scaffolding. Sprint 04 connects it to outcomes with:

- `persona_id`
- `hypothetical_stance`
- optional hypothetical result
- support/block flags
- `evaluation_pending`

This is enough for future offline review without adding a counterfactual search engine.

## How denied trades are evaluated

When Risk Governor denies a Chair-approved candidate:

- the trade is not executed
- a hypothetical triple-barrier outcome may still be computed
- avoided loss generates defensive attribution
- missed gain is tracked as opportunity cost, not catastrophic failure

## How persona votes feed evaluation

`build_persona_evaluation_inputs` aggregates `OutcomeRecord`s by `persona_id` and produces:

- `sample_count`
- `high_confidence_miss_count`
- silence-value contribution
- drawdown / expectancy inputs
- doctrine violation passthrough
- deterministic `SurvivalScoreComponents`

That closes the loop from replayed outcomes back into the Sprint 03 promotion/relegation engine.
