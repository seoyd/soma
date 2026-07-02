# Owner Implementation Goal

## Long-term direction

Soma Zero is intended to become a self-learning AI automated trading system.
Its agents must remain independent, apply distinct investor-style doctrines,
analyze evidence, learn from outcomes, and propose trades. Each agent receives
numeric rewards or penalties from paper outcomes.

The Chair AI is the meta-controller. It manages speaking rights,
promotion/relegation, rewards, penalties, and synthesis, but it cannot execute.
The long-term Chair may improve through numeric outcome feedback, but current
runtime Chair mutation is forbidden.
The system starts with three agents and may expand carefully to eight after
evidence and safety gates pass.

Target markets are US stocks, Korean stocks, and crypto with particular focus on
BTC. Capital preservation is more important than profit. Reckless investing and
overtrading are prohibited.

Owner input is advisory rather than absolute. When Chair or Risk Governor
rejects owner input, the system must return stable reasons explaining why.

## Current binding constraints

- Runtime LLM use in the trading loop is forbidden.
- Every simulated or future live decision must be numeric.
- Default action is `NoTrade`.
- Risk Governor has absolute veto over Chair and every agent.
- Paper/read-only operation must precede any live consideration.
- Toss integration begins read-only.
- Runtime self-mutation and online learning remain forbidden.
- The three-agent system remains active; eight-agent expansion is deferred.

This document locks direction and constraints. It does not activate new agents,
training, real orders, or live trading.
