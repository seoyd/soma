# Trinity Operational Loop

Sprint 56 keeps the existing active Trinity committee and wires it into a deterministic paper-only loop.

## Scope

- reuse the existing 3 active personas
- reuse Chair v0 and Risk Governor
- keep owner review audited and local-only
- keep KIS market-data-only
- keep Control Tower monitor-only

## What the loop does

1. load local evidence-derived candidates
2. classify official / research-only / diagnostic-only / crypto-only boundaries
3. run Trinity persona scoring for official candidates only
4. run Chair review
5. run Risk Governor review
6. optionally queue owner review
7. optionally open simulated paper positions
8. emit deterministic audit and monitor panels

## Safety rules

- no live trading
- no broker, order, account, balance, holdings, or execution APIs
- no runtime LLM
- no Mamba runtime
- no 6/12/18 persona activation
- PaperApproved never becomes a real order
- PaperPositionOpen never becomes a broker position

## Why Trinity stays active

The 3-person Trinity already exists and is the only active committee. Sprint 56 operationalizes it; it does not expand or replace it.
