# Acceptance Truth Gate V7

Acceptance truth gate v7 preserves the core rule: focused pass is not full pass, no-run is not full pass, and verification is not full pass.

`FullWorkspaceAccepted` is valid only when the real full workspace command finished and passed. Anything else must remain blocked, open, still running, failed, or not run.
