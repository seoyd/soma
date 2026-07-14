# Momentum Shadow Walk-Forward Learning

## Purpose

This is an offline, deterministic learning campaign for the experimental frozen Mamba momentum shadow path. It trains only a logistic head and produces evidence records, never a trading decision.

## Historical Evidence

Accepted inputs are immutable normalized daily OHLCV snapshots or sanitized local replay snapshots. Each must be read-only, sanitized, credential-free, quality accepted, digest verified, chronological, finite, and part of one symbol series. Mock, mutable, unsafe, malformed, duplicate-timestamp, and incompatible inputs are rejected. No accepted input yields a no-evidence result with no model version or test metric.

## Windows And Leakage Controls

The implemented policy is an expanding chronological window. Train is followed by a purge gap, validation, another purge gap, and untouched future test rows. The minimum purge gap is `sequence_length - 1 + prediction_horizon`. Each window records ranges and snapshot identifiers. A fresh normalizer fits train feature rows only; validation and test only transform with those frozen statistics.

## Learning Paths

Cold starts derive their seed from campaign, window, and path identity. Warm starts require the immediately preceding compatible shadow version and validate feature, encoder, head, chronology, and deployment boundaries. Test outcomes never select a checkpoint or a parent. The frozen encoder digest must be identical before and after each path.

## Evaluation

Mamba, constant-probability, and linear-momentum paths use the same future test samples. Per-partition diagnostics include Brier score and probability distribution statistics. Aggregate Mamba and warm-start evidence remain insufficient until the configured count of windows is available; a single favorable window cannot establish success. Drift classification is report-only.

## Version And Deployment Boundary

Every accepted path seals a deterministic shadow-only version with campaign/window/path metadata, digests, ranges, snapshot identifiers, metrics, baseline comparison, backend fallback record, and blocked official-conformance status. Generated assessments have voting and execution disabled. There is no promotion, committee membership change, live learning, network fetch, GPU training, or real order path.

## Official Conformance

The encoder is an experimental internal reference. Official Mamba numerical conformance remains blocked until a supported oracle environment is available.
