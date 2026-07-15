# Toss Historical Provider Foundation

The repository's public Toss policy deliberately does not contain a reviewed
historical endpoint, response schema, timestamp semantics, pagination contract,
or Korean/US market field mapping. Accordingly, neither Korean-equity nor
US-equity historical daily OHLCV is qualified by this source tree.

The existing Toss code remains read-only and paper-only guarded. No historical
adapter, guessed request mapping, credentials, live call, order surface, or
account operation was added. A future adapter may be added only after a local
reviewed contract provides the exact capability separately for each market and
a sanitized fixture proves the neutral mapping offline.
