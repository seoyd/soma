# Sprint 124 Three Member Pilot

Sprint 124 starts the committee with three independent AI members instead of one central MoE-style model.

- `TrendEntryAI` looks for entry proposals.
- `RiskGuardAI` looks for volatility, liquidity, drawdown, and no-trade risk.
- `EvidenceRegimeAI` checks evidence quality and regime sufficiency.
- Each member keeps a separate role, deferred Mamba3 + Gated DeltaNet core spec, memory state, score, voice weight, and offline learning journal.
- Paper outcome feedback updates score, voice, memory, and journal entries only; it does not train or mutate model weights.
- The program still routes and orchestrates; AI members judge, Chairman synthesizes, and Risk Governor can veto.
- No broker/order/account, live trading, live inference, model training, runtime LLM debate, real Mamba3 runtime, or real Gated DeltaNet runtime is added.
