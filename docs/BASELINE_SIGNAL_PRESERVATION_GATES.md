# BaselineSignal Preservation Gates

Sprint 96 keeps the BaselineSignal family under the same conservative gates that existed before reduction.

- `NoTrade` stays the default action
- poor-data-quality inputs stay denied
- Risk Governor stays an absolute veto
- source classes remain separated and cannot be promoted implicitly
- no-lookahead stays explicit: future outcomes and labels do not enter the signal path
- interpretation remains research-only, offline, and paper-only

BaselineSignal reduction therefore means test/compile-surface reduction only. It does **not** enable live signals, training, runtime inference, or broker/account paths.
