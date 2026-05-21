# Evidence readiness matrix

Sprint 31 adds a matrix across market, use-case, and source-kind.

## Cell meaning

- `MissingAuth`: provider is relevant but not configured
- `MissingApproval`: provider requires approval before official evidence can run
- `MissingEntitlement`: provider is present but freshness/coverage/endpoint requirements are not satisfied
- `ResearchOnly`: supplemental lane only, never official readiness
- `ReadyForCollection`: bounded lane can be attempted
- `ReadyForBenchmark`: collection and preflight are acceptable for benchmark
- `Evaluated`: research-only benchmark path ran

## Key interpretation rules

- yfinance always has `official_readiness_eligible=false`
- Upbit-only readiness does not imply Korean or US equity readiness
- an EOD-ready cell does not satisfy realtime use-cases
- evaluated means a research benchmark ran, not that live trading is allowed

