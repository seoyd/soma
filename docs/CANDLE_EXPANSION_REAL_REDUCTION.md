# Candle Expansion Real Reduction

`CandleExpansionRealReductionConfig` is the CandleExpansionOps-only control surface for Sprint 89. It validates local-only paths, fixes the target family to `CandleExpansionOps`, and keeps assertion, source-boundary, no-lookahead, storage-budget, and missing-auth preservation explicit.

The associated plan and report separate three concerns:

1. grouped-suite verification,
2. assertion migration accounting,
3. honest queue advancement without equating candle reduction to full workspace acceptance.
