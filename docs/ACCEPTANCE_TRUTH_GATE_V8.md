# Acceptance Truth Gate V8

Acceptance truth gate v8 keeps the acceptance contract explicit:

- focused pass is not full pass
- no-run is not full pass
- CLI smoke is not full pass
- verification is not full pass

`FullWorkspaceAccepted` is allowed only when the full workspace run actually finished and passed.
