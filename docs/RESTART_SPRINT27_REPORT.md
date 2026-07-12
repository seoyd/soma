# Restart Sprint 27 Report

## Backend Result

Added a model-local CPU/Metal/CUDA backend contract. CPU is the complete portable reference backend. Metal is an optional macOS transition pilot only. CUDA remains an unavailable target-gated contract.

## Metal Result

Metal runtime shader compilation implements a paired transition using supplied decay, cosine, sine, state, current contribution, and trapezoidal contribution. It is not routed to full inference.

## Boundaries

Default builds require no GPU dependency. Backend selection is capability-driven with Auto CPU fallback and Strict failure. Official oracle status remains blocked, and no agent/trading integration changed.
