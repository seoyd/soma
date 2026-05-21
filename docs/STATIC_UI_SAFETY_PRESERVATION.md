# Static UI safety preservation

Sprint 94 preserves the existing static UI contract:

- static HTML only
- local-only output paths
- read-only semantics
- paper/research-only presentation
- no POST/forms
- no browser execution
- no order/account/trade controls
- no external remote assets

DashboardRenderer remains a static renderer. It is not a live UI runtime.
