# Workspace Final Gate V2

Sprint 84 keeps the final workspace gate honest.

- full acceptance requires `full_workspace_finished=true` and `full_workspace_passed=true`
- if the full workspace stays long-running, the gate remains `FullWorkspaceStillBlocked`
- focused suites cannot become full acceptance by themselves
- no fake pass/fail is allowed
- safety coverage must remain preserved for any non-failed gate state

