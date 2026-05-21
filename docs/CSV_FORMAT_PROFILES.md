# CSV format profiles

Sprint 17 recognizes these local CSV profiles:

## GenericOhlcv

Required columns:

`timestamp_ms,open,high,low,close,volume`

Common optional columns:

`trade_value,bid,ask,spread_bps`

## Binance-like

Required columns:

`open_time,open,high,low,close,volume`

Common optional columns:

`quote_asset_volume,close_time,number_of_trades,taker_buy_base_asset_volume`

## Upbit-like

Required columns:

`timestamp_ms,opening_price,high_price,low_price,trade_price,candle_acc_trade_volume`

Common optional columns:

`candle_acc_trade_price,market`

## KRX-like

Required columns:

`timestamp_ms,open,high,low,close,volume,trade_value`

## Custom column map

If your header does not match the built-in profiles, provide:

- timestamp
- open
- high
- low
- close
- volume

## Ambiguity behavior

- exact deterministic match => accepted
- low-confidence or conflicting partial match => conservative warning/failure
- strict mode rejects ambiguous/low-confidence detection
- the detector never silently picks among conflicting mappings
