# Restart Sprint 29 Report

## Baseline and Source Audit

The baseline started at `9aaadfc` with default and Metal-feature tests passing. Existing Tiny Mamba SISO, backend selection, stable hash, and frozen evidence types were reused. No active committee, Chair, Risk Governor, PaperBroker, acquisition, or runtime CUDA path was modified.

## Learning Architecture

The implementation adds an experimental frozen Tiny Mamba encoder with a trainable logistic Brier head. It is CPU-only, deterministic, and shadow-only. The official CUDA oracle remains blocked and is recorded in every model version.

## Data and Evaluation

The module provides configurable momentum features, train-only normalization, chronological sequence labels, purge-gap splits, deterministic SGD, validation checkpoint selection, split metrics, constant and linear baselines, and a computed Mamba representation value status.

## Version and Shadow Assessment

Immutable versions record digests, snapshot identifiers, ranges, metrics, backend, experimental mathematical status, and blocked oracle status. The shadow adapter requires frozen evidence and cannot vote or execute.

## Test and Isolation Result

Focused unit tests cover feature leakage, purged chronology, analytical gradients, SGD loss reduction, backend selection, version uniqueness, frozen encoder determinism, and shadow-only eligibility. Final workspace verification is recorded with the implementation commit.

## Remaining Limits

No official Mamba parity, end-to-end Mamba backpropagation, GPU training, market-edge claim, live data training, promotion, voting, or execution capability is claimed. A controlled official CUDA environment and independently reviewed out-of-sample evidence remain required before any promotion discussion.
