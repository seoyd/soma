# Upbit Real Historical Backfill

The sole provider is the existing public Upbit daily-candle operation. A real request requires an ignored local configuration with enabled provider, manual smoke enabled, manual network consent, valid bounded range, page limits, and the explicit CLI network flag. No credential, account, order, cancel, transfer, WebSocket, or background path exists.

The process is single-page first. The initial page must parse, normalize, write atomically, reload, and pass snapshot ID and canonical digest verification before historical pagination is considered. The backfill cursor is the actual oldest normalized candle timestamp and is passed as the endpoint's exclusive `to` boundary.

Backfill stops at the configured row target, calculated campaign threshold when enabled, start boundary, empty/failing page, repeated cursor, repeated page digest, or configured maximum page count. Pages are sorted by timestamp. Identical timestamp duplicates are deduplicated; conflicting OHLCV duplicates reject the merged result. No missing day is fabricated.

The final merged dataset remains a credential-free immutable snapshot with approved-provider provenance. Its normalized payload is stored atomically, reloaded, and digest-verified. Page receipts retain only request ID, receipt ID, cursor, row count, retry count, and snapshot ID; raw payloads and private paths are never retained in reports.

Historical inventory and frozen evidence-pack verification happen after acquisition. No provider call is allowed once evidence is frozen. Campaign execution remains ShadowOnly, requires the existing walk-forward sufficiency gate, and cannot vote, execute, promote a model, or trigger paper/live trading.
