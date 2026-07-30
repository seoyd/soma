# Sprint 103 M3-Micro Three-Agent Report

## 1. Mode and scope

`IMPROVE`, offline historical research only. This sprint introduces three
independent, trainable Soma-specific `M3-Micro` agents. It does not implement
official Mamba-3, modify a live/prospective roster, open a sealed holdout, or
grant trading, Chair, voting, reward, penalty, automatic mutation, or automatic
promotion authority.

## 2. Repository state before change

The repository already contained:

- a portable Mamba-3 SISO reference experiment in `src/model/mamba3.rs`;
- a frozen-encoder plus trainable Logistic-head learning boundary in
  `src/model/learning.rs`;
- T10 historical C0-C4 screening in
  `src/model/momentum_t10_micro_screening_v1.rs`;
- train-only normalizers, chronological splits, leakage guards, deterministic
  hashing, serialization, and verified atomic artifact persistence;
- Default CPU and `backend-metal` feature boundaries, but no Candle dependency.

The working tree already contained untracked `work.md` and `untitled.md`. They
are user-owned and are not modified by this implementation.

## 3. Architecture decision

The implementation lives in one new `src/model/m3_micro.rs` module and is
exported by the existing `src/model/mod.rs`. It reuses existing raw candle,
determinism, chronology/leakage, serde, and atomic persistence boundaries.

Each agent owns a complete model. There is no shared encoder, backbone,
representation, trainable tensor, recurrent state, optimizer state, or learned
normalizer. Immutable market evidence, formula specifications, causal formula
results, artifact helpers, and evaluation utilities may be shared.

`M3-Micro` is a Soma-specific experimental recurrent core inspired only by
selective state-space principles. It is not an official Mamba-3 implementation
or parity claim.

### Frozen tensor contract

Default per-agent configuration:

```text
input formula vector x_t          [8]
input embedding h_t^0             [64]
block count                       2
block expansion                   2
block inner width                 128
block recurrent state             [128, 8]
block previous projected input    [128]
block output                      [64]
Trend raw output                  [5]
Volatility raw output             [5]
Reversal raw output               [6]
```

For block `l`, with `u_t`, gates, and state indexed by inner channel `c` and
state coordinate `s`:

```text
u_t = tanh(W_in h_t^(l-1) + b_in)

decay_t[c,s] =
  decay_min + (decay_max - decay_min)
  * sigmoid((W_decay u_t + b_decay)[c] + decay_state_bias[c,s])

prev_gate_t = sigmoid(W_prev_gate u_t + b_prev_gate)
curr_gate_t = sigmoid(W_curr_gate u_t + b_curr_gate)

state_t[c,s] =
    decay_t[c,s] * state_(t-1)[c,s]
    + prev_gate_t[c] * previous_u[c] * tanh(prev_scale[c,s])
    + curr_gate_t[c] * u_t[c] * tanh(curr_scale[c,s])

readout_t[c] =
    sum_s(state_t[c,s] * tanh(readout_scale[c,s])) / d_state

z_t[c] =
    tanh(readout_t[c] + sigmoid(skip[c]) * u_t[c])

h_t^l = tanh(W_out z_t + b_out)
```

The recurrence is linear in sequence length. `decay_min = 0.01` and
`decay_max = 0.98`; gates, injection scales, skip, readout, block outputs, and
stored parameters are bounded. Non-finite input, gradient, parameter, output,
or recurrent state fails closed. State magnitude above the explicit state
limit also fails closed.

The implementation uses deterministic scalar Rust `f32` operations and manual
reverse-mode BPTT through the complete recurrent core and prediction head,
within the repository's existing verified Rust learning boundary. No new
runtime or network dependency is introduced.

## 4. Reused components

- canonical `Candle`/`CandleSeries` evidence;
- `FeatureEngine`-compatible causal OHLCV semantics;
- `stable_hash_string` deterministic identities;
- existing chronology/leakage concepts;
- serde JSON reconstruction;
- existing verified atomic artifact writer;
- existing Default/Metal Cargo feature boundary and serial test policy.

## 5. Added components

- deterministic `FormulaRegistry`, immutable causal `FormulaResultCache`, and
  three ordered agent Formula Genomes;
- bounded two-point recurrent M3-Micro core with complete reverse-mode BPTT;
- independent Adam state, train-only normalizer, recurrent state, histories,
  identity fields, and three-head role policies per agent;
- Trend distribution/continuation/return loss, Volatility QLIKE/regime/risk
  loss, and Reversal event/return/direction loss;
- manual Formula mutation challenger and fail-closed manual promotion boundary;
- isolated JSON checkpoint reconstruction using the existing verified atomic
  artifact writer;
- development/validation-only runner with persisted prediction-before-reveal,
  C0/agent-specific baseline comparison, and no holdout partition type;
- Brier, RPS, and QLIKE evaluation utilities, safety counters, resource
  measurement, and focused unit/synthetic tests.

## 6. Legacy Logistic boundary

C1-C4 remain readable only as immutable historical T10 evidence. The new
roster, challenger creation, training, evaluation, selection, and promotion
types cannot represent a Logistic candidate. Their explicit disposition is:

```text
LegacyHistoricalBenchmarkOnly
NotActiveAgent
NotPromotionEligible
NotProspectiveCandidate
```

C0 remains a non-learning prevalence/constant evaluation baseline and is not
an agent.

## 7. Three-agent independence proof

The roster contains exactly three separately allocated agent values. Each owns
its parameters, recurrent state, optimizer moments, normalizer, Formula
Genome, schema and policy identities, histories, checkpoint/artifact identity,
and promotion history.

Focused tests proved:

- one Trend training step changed only Trend parameter and optimizer digests;
- one Trend recurrent inference changed only Trend state digest;
- deleting/corrupting/restoring the Trend checkpoint neither read nor changed
  Volatility or Reversal checkpoints;
- a Trend Formula challenger changed only its schema/model identities;
- all three parameter buffers, six optimizer buffers, six normalizer buffers,
  and twelve recurrent state/previous-input buffers have distinct storage
  addresses.

## 8. M3-Micro equation and tensor contract

The frozen equation and shapes are specified in section 3. Any future tensor
shape, Formula set, target policy, or loss-policy change must create a fresh
model/schema identity.

## 9. Formula Registry and Genome

The registry records formula id/version, required sources/history, output
dimension, normalization, finite fallback, causal-only flag, and cost class.
Each agent owns an ordered eight-formula Genome. Formula results may be cached
as immutable causal values; normalizer or learned state may not enter that
cache. Order-flow formulas are registered but rejected as
`UnavailableSourceEvidence` when canonical evidence does not contain that
source.

Formula mutation is manual and deterministic. It creates a fresh challenger
with a new schema digest, initialization, optimizer, normalizer, state, model
identity, checkpoint identity, and artifact identity. The champion is not
modified before an explicit eligible promotion.

## 10. Parameter and memory budget

Measured on macOS Apple Silicon (`aarch64`, Rust `f32`, debug test profile) with
one four-step `[4,8]` input per agent:

| Agent | Parameters | JSON checkpoint | Recurrent state | Inference |
| --- | ---: | ---: | ---: | ---: |
| Trend | 141,573 | 2,875,776 B | 9,224 B | 16,592,125 ns |
| Volatility | 141,573 | 2,876,376 B | 9,224 B | 16,760,041 ns |
| Reversal | 141,638 | 2,878,018 B | 9,224 B | 16,268,583 ns |

Total parameters are 424,784. Sequential three-agent inference was
49,620,749 ns. A shared-model batch is intentionally unsupported. The
deterministic owned-tensor estimate is 2,274,392 B for one Trend/Volatility
training step, 2,275,432 B for one Reversal training step, and 1,726,808 B for
three-agent inference. These are owned tensor/state estimates, not process RSS
measurements. The populated 24-entry Formula cache measured 1,380 B by its
explicit key/value accounting.

All agents are in the 100,000-300,000 target range and below the 500,000 hard
maximum. Latency is a single debug-profile observation, not a performance
improvement claim.

## 11. Synthetic learning evidence

The focused Default and Metal suites passed strict `final_loss < initial_loss`
checks for:

- delayed signal recall where only the first step contains the signal;
- Trend continuation;
- Volatility regime expansion/switch;
- Reversal/distortion.

Separate core tests passed single-impulse memory, two-point sensitivity,
selective forgetting, 256-step finite boundedness, zero/constant input,
deterministic replay, and fail-closed numerical rejection. These tasks prove
only that optimization and the recurrent computation work.

## 12. Historical integration status

The integration accepts development and validation examples only, fits
normalizers and parameters on development only, persists each validation
prediction before target reveal, and exposes no sealed-holdout partition.
Until an explicit offline historical run is executed, status is
`ImplementationCompleteEvidencePending`.

The focused fixture completed all three development/validation lanes, wrote
one target-free validation prediction artifact per agent before reveal, fitted
only on development rows, compared each result to its training-prevalence C0
and role-specific mathematical baseline, and kept legacy Logistic results
read-only/unexecuted. No real market development/validation result was run or
claimed.

## 13. Default/Metal verification

All Rust commands used `CARGO_BUILD_JOBS=1` and never overlapped:

- `cargo check --lib`: passed; one new unused import was found and removed;
- Default focused suite: 13 passed, 0 failed;
- `backend-metal` focused suite: 14 passed, 0 failed, including exact portable
  reference architecture/state/output parity;
- `cargo fmt --all`: passed;
- `git diff --check`: passed.

The only reported test warning is the pre-existing dead-code warning for
`train_encoded_head` in `learning_campaign.rs`. Metal support here means the
same portable architecture and parameter contract compiled under
`backend-metal`; no Candle dependency existed before the change, and no Metal
custom kernel or acceleration claim is made.

## 14. Safety counter verification

All Sprint 103 counters are structurally initialized to zero and validated
before/after historical research execution. Focused tests kept network, live
market, sealed holdout, live prediction/outcome, paper/live trade, order,
account, winner-selection, Chair, voting, reward/penalty, automatic promotion,
automatic Formula mutation, and prospective-state-write counters at zero.

The aggregate SHA-256 identity of existing paths containing
`prospective|live|holdout` was
`9624ca230998a5fa761923a52a426a892b8e867c0e809fdde1121d70ba4c4703`
before and after implementation.

## 15. What this proves

The permitted claims are limited to:

- M3-Micro operates as a trainable lightweight time-series core;
- three agents have independent model, state, and learning boundaries;
- a Formula challenger replacement boundary exists;
- existing Soma safety and evidence boundaries remain preserved.

## 16. What this does not prove

It does not prove market predictive improvement, official Mamba-3 equality,
superiority to Logistic models, profitability, live-investment readiness, or
autonomous agent evolution.

## 17. Remaining risks

- A real preregistered historical development/validation run remains pending.
- The portable scalar backend has no Candle autograd or Metal acceleration;
  Metal evidence is feature/build/reference parity only.
- Formula definitions remain research formulas and require historical
  evaluation; their existence is not predictive evidence.
- Memory figures are deterministic owned-tensor estimates rather than measured
  process peak RSS.
- Focused changed-scope tests passed; the user-requested test scope explicitly
  excluded the full repository suite.

## 18. Exactly one next recommended step

Run one preregistered, development/validation-only historical evaluation
without opening the sealed holdout.
