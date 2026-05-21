# Sprint 25 report

## Implemented items

- provider auth preflight config, runner, report, and deterministic renderers
- venue coverage target plan and auth-aware coverage report
- official evidence expansion runner with core-check-gated benchmark rerun
- official evidence delta and storage delta summaries
- auth setup guides, CLI wiring, example configs, and Sprint 25 tests

## Tests

- provider auth preflight behavior and secret-safety coverage
- venue coverage target status and determinism
- official evidence expansion config, runner, and CLI safety
- evidence delta, storage delta, and auth setup guide checks

## Provider and auth status

- Upbit remains public and `NotRequired`
- KRX remains blocked until a key env var and endpoint template env var are present
- AlphaVantage remains blocked until an API key env var is present
- Alpaca remains optional or deferred for Sprint 25

## Coverage status

- Upbit-only evidence is explicitly labeled `CryptoOnly`
- Korean and US equity claims remain blocked when auth or official coverage is missing
- mock fixtures and non-official inputs stay excluded from official readiness claims

## Data-size status

- bounded storage delta reporting is in place
- compact examples stay within small symbol and row budgets
- compaction guidance is emitted before budget overrun turns into silent growth

## Risk review

- no live trading path was added
- no broker, order, or account API path was added
- no runtime LLM or Mamba runtime was added
- core-check still gates benchmark execution
- official evidence breadth is still not a profitability claim

## Next sprint recommendation

Stay in bounded research mode and expand official multi-venue evidence conservatively before any model, sequence, or deployment scope change.
