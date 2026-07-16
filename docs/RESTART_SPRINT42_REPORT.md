# Restart Sprint 42 report

The project now has a constrained local Toss contract-intake boundary for one
daily OHLCV capability at a time. It parses only a typed ignored manifest,
validates required historical semantics, computes a deterministic semantic
digest, qualifies Korean and US capabilities independently, and selects at
most one qualified capability with Korean preference.

No local Toss contract material, execution configuration, or consent was
available during this sprint. Therefore both equity capabilities report
`ContractMaterialUnavailable`; no adapter mapping or fixture was invented, and
no Toss network request, snapshot, inventory, evidence pack, or equity campaign
occurred.

Existing BTC evidence and ShadowOnly isolation remain unchanged. The
three-market matrix continues to retain the two blocked equity rows.
