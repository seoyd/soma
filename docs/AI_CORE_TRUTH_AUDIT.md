# AI Core Truth Audit

## Audit Basis

This audit classifies executable source separately from plans, reports, fixtures,
and deferred contracts. The repository is one Cargo package and one workspace
member. `Cargo.toml` disables automatic integration-test discovery and registers
two explicit integration-test targets.

The latest completed baseline before this audit passed formatting, workspace
checking, and 459 tests. This audit does not treat a compiled type, a report
generator, or a readiness contract as proof that a learning model exists.

## Status Summary

| Area | Status | Source truth |
| --- | --- | --- |
| Safety paper loop | Implemented | `backtest::simulate_paper_cycle` |
| Three policy delegates | Implemented | `momentum_trend_fast`, `value_quality_filter`, `cycle_risk_skeptic` |
| Independent learning agents | Partial | Independent rules and votes exist; learned model policies do not |
| Chair decision logic | Implemented/partial | Speaker selection and synthesis exist; active reward governance does not |
| Risk Governor | Implemented | Deterministic hard gates and paper-only approval |
| Reward/penalty evaluation | Partial | Pure scoring and offline/shadow updates exist |
| Persistent agent learning | Partial | Canonical versioned summaries now exist; persistence and policy fitting do not |
| Heavy signal model | Missing | Mamba/Gated DeltaNet runtime and training are deferred |
| Toss integration | Sanitized fixture only | Mock transport and fabricated mapping |
| Real trading | Missing by policy | No live broker or order path |

## A. Implemented

### Canonical Paper Safety Path

The concrete paper path is:

```text
MarketSnapshot
-> deterministic feature derivation
-> MockSignalEngine
-> momentum_trend_fast
-> value_quality_filter
-> cycle_risk_skeptic
-> ChairEngine
-> RiskGovernor
-> PaperBroker
```

`src/backtest/simulator.rs` owns this sequence. The Chair can construct only a
candidate proposal. `RiskGovernor::evaluate` independently denies, cools down,
emergency-stops, or emits a paper-only `OrderPlan`. `PaperBroker` is the only
`Broker` implementation and reports that live execution is unsupported.

### Numeric Policy Delegates

The three active delegates have separate doctrine cards, horizons, mutable
thresholds, voice power, votes, and style-specific logic:

- `momentum_trend_fast`: short-horizon trend and momentum policy.
- `value_quality_filter`: defensive filter with no intraday-entry role.
- `cycle_risk_skeptic`: cycle/risk policy with defensive veto behavior.

They are independent policy delegates, not trained learning models.

### Chair And Risk

`ChairEngine` filters voices by horizon, ranks speakers, introduces a
contrarian, penalizes clustered votes, measures disagreement/groupthink, and
defaults to `NoTrade`. It does not execute.

`RiskGovernor` is a separate final gate. It checks daily loss, loss streak,
edge, confidence, stop/take-profit, spread, liquidity, data/API quality, regime,
volatility, risk/reward, and exposure. Its approved plan is always paper-only.

### Evaluation And Governance Primitives

The repository has deterministic primitives for:

- immutable doctrine and mutable persona policy,
- doctrine violations and quarantine,
- survival scoring and silence value,
- bounded voice updates,
- promotion, demotion, and tier classification,
- paper outcome attribution and backtesting,
- owner advisory validation and stable rejection explanations.

### Offline Committee Experiment

`src/league/minimal_ai_committee_core.rs` contains a second, experimental lane:

- member identities and roles,
- local routing and committee sessions,
- deterministic mock or offline-fixture opinions,
- paper outcome feedback,
- score/voice update records,
- memory summaries and learning journals,
- shadow-only Chair reward/penalty candidates,
- deferred core contracts and safety guards.

This lane explicitly reports no model training, weight update, checkpoint, live
inference, broker, account, or real order.

### Data And Test Surfaces

The repository includes feature extraction, regime classification, backtests,
triple-barrier outcomes, calibration, external prediction-file import, and
local/official market-data collection tooling. US and Korean market-data
components exist in varying readiness states. Upbit candle collection and
Binance/Upbit CSV formats exist, but this does not make a unified production
crypto adapter part of the canonical paper loop.

The registered test surface covers the large minimal committee suite, workspace
recovery checks, and library unit tests including Toss safety. Because
`autotests = false`, unregistered files under `tests/` are not proof of executed
coverage.

## B. Partial

### Independent Agents

Agents have distinct doctrines, votes, thresholds, horizons, and evaluation
profiles. The active paper path does not persist a unique learned policy or
memory per delegate. Independence is therefore policy-level, not learned-model
independence.

### Learning

Offline outcome feedback updates score/voice records and memory counters using
fixed deltas. Learning journals label reinforce/penalize/watch/ignore. These are
useful controlled feedback primitives, but they do not fit policy parameters,
train a model, create validated child variants, or promote a new agent version.

Sprint 07 adds a canonical paper-only state, memory update, Chair
reward/penalty, parent-linked state version, and sandbox candidate metadata.
These are pure in-memory contracts. They do not yet persist versions, adapt the
active paper loop, search policy parameters, or activate a child version.

### Chair Governance

The active Chair selects and synthesizes speakers. Extensive shadow contracts
and ledgers model reward, penalty, voice, and promotion candidates, while actual
score/voice/promotion mutation remains disabled. Chair governance is designed
and simulated, not active learning governance.

### Promotion And Relegation

Pure tier functions and evaluation rules exist. The offline committee also
produces promoted/demoted flags. There is no single canonical, persisted,
validated deployment process that changes an active agent version.

### Market Adapters

Korean and US data collectors/importers are broader than the Toss work, but
their readiness differs by provider and they are not one unified input contract
for the canonical three-agent loop. Crypto support is strongest in local CSV
and bounded Upbit candle collection, not a complete multi-market runtime.

## C. Stubs And Placeholders

- `MockSignalEngine` is deterministic feature arithmetic, not an AI model.
- `BaselineSignalModel` is a rule-based baseline.
- `DeterministicMockBrain` is explicit mock logic.
- `OfflineMemberBrainAdapter` replays fixtures and falls back to
  `NeedMoreEvidence`.
- Mamba/Gated DeltaNet core specifications and readiness reports are contracts,
  not model implementations.
- Toss paths and quote mapping are fabricated sanitized contracts, not official
  provider mappings.
- Toss live smoke remains documentation-only.

## D. Missing

- A runtime adapter from the canonical learning-agent interface into the paper path.
- Per-agent persisted policy versions and active outcome ledgers.
- Outcome-driven policy fitting with bounded, validated parameter changes.
- Sandbox child creation, comparative evaluation, and promotion into a stable
  next version.
- Mamba3/Gated DeltaNet signal or memory runtime.
- A trained model backed by a validated real dataset.
- Eight active, validated investor agents.
- Official Toss read-only endpoint and field mapping.
- A unified production-grade crypto input adapter.
- A unified US/Korean provider-to-agent runtime path.

## E. Deferred

- Mamba3, Gated DeltaNet, Sparse Mamba, FA3, and routing models.
- Neural-network training and online learning.
- Live self-evolution and live mutation.
- Full eight-agent activation.
- Real broker, order placement, cancellation, and live trading.
- Runtime LLM, web UI expansion, and cloud deployment.

## F. Conclusion

**AI Core Status: PARTIAL**

The repository has a substantial deterministic committee, evaluation, and
offline feedback foundation. It does not yet have true outcome-trained,
independently versioned learning agents.

**Safety Foundation Status: VERIFIED**

The latest completed baseline verified the paper-only path, Risk Governor veto,
mock Toss transport, deterministic tests, and absence of a live broker path.

**Toss Read-only Status: SANITIZED_FIXTURE_ONLY**

The client and parser operate against mock transport and fabricated schemas.
They are not local-smoke-ready or live-ready.

**Trading Readiness: NOT READY**

The next highest-leverage step is to adapt completed paper outcomes into the
new canonical feedback contract, persist versioned paper-only state, and prove
sandbox evaluation gates before adding any heavy model.
