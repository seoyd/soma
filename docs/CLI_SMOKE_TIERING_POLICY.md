# CLI Smoke Tiering Policy

Sprint 106 separates CLI smoke into three layers:

1. representative smoke
2. exhaustive smoke
3. safety smoke

Safety smoke remains separate on purpose. It protects acceptance truth, safety coverage, and read-only UI boundaries without being diluted by broader diagnostic commands.
