# KRX genuine reduction gate

`KrxEvidenceReducedWithWarnings` from Sprint 91 is not the same as genuine closure.

## Genuine vs warning-backed

- warning-backed: at least one remaining manual-review or safety interpretation gap exists
- reduced with isolated sentinel: all required preservation gates pass, but the raw archive sentinel remains explicitly isolated
- genuinely reduced: no remaining isolated sentinel and all preservation gates pass

## Manual review closure

Sprint 92 closes manual review as `ManualReviewClosedWithIsolatedSentinel` because the remaining warning is preserved and explicitly justified.

## Raw archive redaction coverage

Raw archive coverage must confirm:

- assertion presence
- auth/header redaction
- secret value non-rendering
- local-only path handling
