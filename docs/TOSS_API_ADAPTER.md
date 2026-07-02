# Toss API Adapter

## Scope

The Toss integration is a read-only, paper-only foundation. `TossClient` exposes a
mock health check and a market snapshot contract. It has no order submission,
cancel, modification, balance, holdings, or live execution API.

No local Toss API specification was available when this sprint was implemented.
The paths under `/soma/read-only/*` are deterministic mock transport contracts,
not claims about Toss production endpoints. There is no real HTTP transport and
no unit test can reach a network.

## Configuration

`TossApiConfig` contains the base URL, credential environment-variable names,
optional account environment-variable name, timeout, retry limit, optional rate
limit, and mandatory `paper_only` and `read_only` flags. Validation rejects
non-HTTPS base URLs, URL query/userinfo/fragment content, unsafe environment
variable names, and either safety flag being disabled.

`TossCredentials::from_env` loads values from the configured environment
variables. Credential fields are private and their `Debug` output is redacted.

## Transport and client

`TossEndpointContract` locks method, path template, required fields, response
schema, read-only status, account scope, and test/smoke permissions. The active
client resolves registered contracts before constructing a request.

Only mock health and quote contracts are callable. Candle and account contracts
validate as read-only shapes but remain disabled because verified private
contracts are not available publicly. Token auth is a disabled `Post`
placeholder and cannot be treated as a trade. Order, cancel, unknown, and every
non-read-only contract are rejected before transport invocation.

`MockTossTransport` provides queued deterministic responses for unit tests.
Response and request bodies are excluded from `Debug` output. There is no real
transport.

## Sanitized schema lock

`TossQuoteFieldMapping` defines the neutral public key mapping used by sanitized
fixtures. Its default keys are `symbol`, `timestamp_ms`, `price`, `bid`, `ask`,
`volume`, `trade_value`, and `raw_status`. This is not claimed to be the
official Toss schema. `PrivateContractMappingRequired` remains attached to the
quote contract until a local private review confirms a sanitized mapping.

`TossQuoteResponse` defines Soma's neutral public response:

- symbol and timestamp
- positive price
- optional bid, ask, spread, volume, trade value, volatility, and raw status

`map_quote_response_to_internal_snapshot` validates the mapping, rejects missing
required fields, non-positive prices, invalid bid/ask, and non-finite values,
computes spread from bid/ask, and produces the existing `MarketSnapshot`.
The `_with_mapping` variant accepts a locally reviewed sanitized mapping without
embedding private documents or raw examples.
Sensitive mapping names return `SensitiveFieldNameRejected`. Empty or duplicate
mapping keys return `MappingValidationFailed`; neither error exposes the
rejected field value.
Malformed data and missing timestamps are structured errors. Stale timestamps
and wide spreads reduce data quality.
Fixtures under `fixtures/toss/` are fabricated and scanned for obvious secrets.
They are not copies of private Toss documentation. Final provider field mapping
requires local private-contract review.

## Data flow

`TossReadOnlyAdapter` maps a parsed quote into the existing `MarketSnapshot` and
`RiskSnapshot` types:

1. Toss read-only input
2. Existing feature/signal path
3. Existing three delegates
4. Existing Chair
5. Existing Risk Governor
6. Existing `PaperBroker`

The adapter does not create a trade proposal or call a broker. API errors produce
zero API health, zero data quality, and default `NoTrade`. Missing price, stale
timestamps, and absent spreads reduce data quality; absent spreads use a
conservative 50 bps value.

## Future requirements

Production Toss endpoints, authentication, candle/account schemas, rate-limit
semantics, and field mapping require local private-contract review and a
separate approved read-only sprint. Private documents stay under ignored
`local_private/` paths and must never be committed. Any future order work
requires a separate explicitly approved sprint, a compile-time feature disabled
by default, new security review, and preservation of the Risk Governor veto. No
such feature exists now.
