# Symbol / Timeframe Registry

Sprint 09 adds a small deterministic registry layer.

## SymbolSpec

`SymbolSpec` tracks:

- raw symbol
- normalized symbol
- asset class
- venue
- optional base/quote currency
- optional tick/lot/timezone metadata
- reason codes

`SymbolRegistry`:

- normalizes symbols locally
- registers known symbols
- looks them up without network access
- validates invalid/empty identifiers conservatively

There is **no** exchange metadata download in this sprint.

## TimeframeSpec

`TimeframeSpec` tracks:

- timeframe enum
- seconds
- expected millisecond step
- gap policy hint
- session awareness hint
- reason codes

It exists to make CSV validation and resampling deterministic before data reaches the existing evaluator path.
