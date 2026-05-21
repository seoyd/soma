# UI Framework Decision

Sprint 60 does **not** migrate the UI stack. It records the decision instead.

Current decision:

1. **now:** static HTML + JSON/TXT
2. **optional:** small vanilla TypeScript/CSS enhancements
3. **later:** Tauri + Svelte, only if richer local desktop interaction becomes justified

Rejected for now:

1. React/Next web app
2. cloud dashboard / remote hosting
3. server-heavy interactive UI

The reasoning is simple: the repo still needs evidence hardening and review ergonomics more than it needs framework churn. Static local output remains the safest choice for the current paper-only stage.
