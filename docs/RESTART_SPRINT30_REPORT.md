# Restart Sprint 30 Report

## Baseline And Source Audit

The implementation reuses the existing frozen encoder, causal feature builder, train-only normalizer, sequence labels, logistic Brier head, SGD, validation checkpointing, baselines, model journal, snapshot contract, backend selector, and shadow boundary. Active committee, Chair, Risk Governor, PaperBroker, acquisition behavior, and runtime network behavior remain unchanged.

## Campaign Implementation

The new campaign module validates immutable sanitized historical snapshots, builds deterministic expanding windows with two purge boundaries, fits a normalizer per train range, trains cold and eligible warm paths, seals test evaluation, records shadow versions, and computes baseline, aggregate, warm-start, and drift evidence. Unsafe or absent evidence produces no accepted version.

## Safety And Limits

All generated versions and assessments are `ShadowOnly` with voting and execution disabled. The encoder is frozen and CPU-only; partial Metal and unavailable CUDA do not become training backends. The implementation makes no official-conformance, profitability, market-edge, promotion, active-agent, or live-learning claim.

## Verification

Workspace formatting, default checks, default tests, and supported Metal-feature checks are run serially. The final implementation report records the exact command outcomes together with the commit used for shipment.

## Next Recommendation

Run the campaign only with independently reviewed immutable local evidence, inspect aggregate out-of-sample outcomes across the configured number of windows, and keep any promotion discussion separate from this shadow-only implementation.
