# Sprint 45 restart report

Offline inspection reproduced the previous conflict: the historical request
cursor targeted the accepted range, producing 200 overlapping timestamps. Of
those, 199 were canonical duplicates and one had a canonical `high` field
difference. The conflicting bar was potentially open at acquisition, but the
primary root cause was the cursor overlap, not a completed-bar reconciliation.

The strict plan derived its exclusive cursor from the oldest accepted row,
requested the exact missing evidence count, proved zero expected overlap, and
forced zero live retries. One authorized sequential public BTC daily request
returned a strictly older validated page. It was appended without overlap to a
new immutable Protobuf V1 snapshot; the original snapshot was not modified.

The expanded evidence met the configured two-regime requirement. Network access
was closed before frozen-pack evaluation. All model policy remained ShadowOnly;
the cross-regime diagnostic result is research-only and does not enable voting,
promotion, execution, or live trading.
