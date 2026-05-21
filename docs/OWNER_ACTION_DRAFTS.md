# Owner Action Drafts

Owner action drafts are local TOML or JSON files generated from Control Tower v1 state.

## Draft vs applied input
- draft: local file suggestion only
- applied input: explicit owner CLI invocation through `owner-apply-input`, `owner-review-queue`, `owner-impact-report`, or `owner-thesis-book`

## Constraints
- paper-only
- no auto-apply
- Risk Governor remains absolute
- RiskBlocked and NoTrade candidates do not receive paper-confirm drafts
- browser UI never executes the draft
