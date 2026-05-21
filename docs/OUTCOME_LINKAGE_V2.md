# Outcome Linkage V2

Sprint 45 turns outcome linkage into explicit backfill planning.

## Included backfills
- triple-barrier outcome reference backfill
- baseline reference backfill
- NoTrade counterfactual backfill
- RiskDenied counterfactual backfill

## Conservative rules
- all backfill planning is local-only and deterministic
- `no_lookahead_safe = false` blocks outcome and counterfactual use
- missing future bars, timestamp mismatch, or horizon mismatch stay explicit
- NoTrade baseline fallback is valid and should not be ignored
- counterfactual backfill cannot promote source class or erase diagnostic boundaries

## Interpretation
Backfillability means the local evidence path exists. It does **not** mean the strategy is profitable, promotion-ready, or live-trading ready.
