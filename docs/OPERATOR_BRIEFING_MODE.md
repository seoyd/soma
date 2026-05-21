# Operator Briefing Mode

Sprint 71 adds a one-screen **static/read-only** operator briefing for local paper/research monitoring.

## What the owner can see

- system health and direct-watch readiness estimate
- owner attention items and blocked items
- leaderboard warning handling and retirement evidence status
- deferred items
- copyable local commands

## What the owner cannot do

- execute trades
- submit browser actions or POST requests
- enable runtime inference
- enable Mamba runtime
- train models
- access broker/order/account controls

The generated briefing stays local-only, deterministic, and conservative. Even when the briefing is ready, it does **not** imply live-trading readiness.
