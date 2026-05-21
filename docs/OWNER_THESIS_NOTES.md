# Owner Thesis Notes

Owner thesis notes are **diagnostics, not signals**.

## Properties

- structured tags
- optional evidence links
- optional expiration timestamp
- active/expired separation in `OwnerThesisBook`

## Safety

- thesis notes do not bypass numeric scoring
- thesis notes do not bypass the Risk Governor
- expired notes stay in audit history but are excluded from active review prompts

## Dashboard behavior

Active notes appear in:
- the owner panel,
- linked candidate rows for the same symbol,
- review context for paper-only owner oversight.
