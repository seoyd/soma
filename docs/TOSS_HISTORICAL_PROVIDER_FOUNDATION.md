# Toss Historical Provider Foundation

The repository's public Toss policy deliberately does not contain a reviewed
historical endpoint, response schema, timestamp semantics, pagination contract,
or Korean/US market field mapping. Accordingly, neither Korean-equity nor
US-equity historical daily OHLCV is qualified by this source tree.

The capability report evaluates Korean and US daily OHLCV independently as
`ContractIncomplete`. The result is fail-closed: no endpoint path, request
mapping, credential lookup, transport, parser, snapshot, or smoke request is
enabled from this status.

The existing Toss code remains read-only and paper-only guarded. No historical
adapter, guessed request mapping, credentials, live call, order surface, or
account operation was added. A future adapter may be added only after a local
reviewed contract provides the exact capability separately for each market and
a sanitized fixture proves the neutral mapping offline.
