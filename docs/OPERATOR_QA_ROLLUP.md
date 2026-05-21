# Operator QA Rollup

Sprint 67 adds a deduplicated **operator QA rollup**.

It collapses repeated raw QA rows into one item per model version and keeps:

- checklist summary
- safe actions
- blocked actions
- required owner actions
- one next command

Next commands remain copy-only local commands. There are no execution buttons and no live/runtime/train actions.
