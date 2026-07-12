# Momentum Mamba Sandbox Agent

## Scope

`FrozenMambaReservoirHeadV0` is an experimental, CPU-only learning path. It uses the existing Tiny Mamba-3 SISO core as a frozen recurrent encoder and trains only a logistic prediction head. It is not a fully trained Mamba-3 model and has no trading, committee, Chair, or Risk Governor authority.

## Architecture

```text
normalized chronological features -> frozen Tiny Mamba-3 encoder -> pooled representation
                                                     -> trainable logistic probability head
```

The only trainable values are the head weights and bias. Mamba projections, transition values, normalization values, recurrent state behavior, and output projection remain unchanged. Training records encoder parameter digests before and after optimization and rejects mutation.

## Features and Leakage Controls

The configurable ordered schema contains log return, medium-horizon momentum, moving-average distance, rolling volatility, drawdown, and volume z-score. Each feature uses only its current and preceding OHLCV rows. Input validation rejects non-finite values, invalid prices, negative volume, and insufficient history.

Normalization is fitted only from training feature rows. Constant columns are transformed with scale `1.0` and recorded explicitly. Validation and test rows are transformed with the fitted training statistics and never alter them.

## Labels and Splits

Sequence examples are chronological. A label is `1.0` when the configured future return exceeds the positive dead zone, `0.0` when it is below the negative dead zone, and neutral rows are excluded by default. Train, validation, and test partitions retain order and require a purge gap large enough to prevent sequence/label overlap. Test data is never supplied to the optimizer.

## Training and Evaluation

The head uses a stable sigmoid and Brier loss with analytical gradients. Deterministic SGD supports explicit weight decay and gradient clipping. Validation selects the best head snapshot; the restored best head is separate from the final epoch head.

Evaluation reports Brier score, sample count, accuracy, positive-label rate, mean probability, and high-confidence errors for each split. The same labels/splits/optimizer are available to a train-only constant probability baseline and a linear raw-feature baseline. The Mamba representation value status is computed as `Helped`, `Failed`, `Mixed`, or `InsufficientEvidence`; it is never predeclared.

## Version and Backend Policy

Each immutable sandbox model version records feature/normalizer/encoder/head/training digests, sorted snapshot identifiers, split ranges, metrics, selected backend, and the blocked official-conformance status. Duplicate IDs are rejected by the in-memory journal.

Inference goes through the existing backend selector and requires `FullInferenceReady`. Current full inference is CPU. Partial Metal and unavailable or contract-only CUDA are rejected; Auto may fall back to CPU with its recorded reason. Training is CPU-only `f32`.

## Shadow Boundary

`MomentumMambaSandboxAgentV0` accepts frozen evidence plus a prepared sequence and emits a deterministic `ShadowAgentAssessmentV0`. Every assessment has `ShadowOnly`, `eligible_to_vote = false`, and `eligible_to_execute = false`. It is not an active proposal and is not included in the three-agent committee.

## Official Conformance Warning

The encoder remains an `ExperimentalInternalReference`. The official Mamba-3 CUDA oracle is still blocked on the current host, so this path makes no official numerical-parity claim.

## Promotion Preconditions

Promotion is outside this implementation. It requires genuine official conformance evidence, repeatable out-of-sample evaluation, sufficient data, a skeptical baseline comparison, explicit safety review, and a separate decision to alter committee membership.
