# Sprint 32 report

## Summary

Sprint 32 adds a minimal three-persona committee MVP with Chair v0, Risk Governor handoff, committee smoke reporting, and a conservative evaluation scaffold.

## Implemented

- `PersonaCardLite`, `PersonaVote`, `PersonaScoringInput`, and three numeric scorers
- Chair v0 with cluster penalty, contrarian inclusion, and deterministic weighting
- committee-to-risk bridge with paper-only outcomes
- committee smoke runner with fixture / Upbit / yfinance research paths
- committee evaluation scaffold
- `committee-smoke` and `persona-cards` CLI
- examples, docs, and Sprint 32 tests

## Committee behavior

- exactly three active personas participate
- yfinance stays research-only
- Upbit stays crypto-only in committee smoke
- high disagreement and groupthink are treated conservatively

## Risk review

- Risk Governor remains absolute
- no live trading path was added
- no broker/order/account command was added
- approved paths still produce paper-only order plans

## Tests

- persona cards
- persona scorers
- Chair v0
- committee risk bridge
- committee smoke runner
- committee evaluation scaffold
- committee CLI safety
- committee determinism

## Next sprint recommendation

Keep Sprint 33 focused on better fixture/evidence ingestion and committee diagnostics before any design review for 6-person expansion.

