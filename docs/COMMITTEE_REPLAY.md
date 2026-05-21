# Committee replay

`CommitteeDebateReplay` replays committee decisions from a scenario set or committee smoke config.

## Guarantees

- deterministic replay fingerprints
- no wall-clock dependency
- no random behavior by default
- no live execution path

## Flow

1. load committee scenarios
2. run the 3 numeric persona scorers
3. run `ChairV0`
4. run `CommitteeRiskBridge`
5. emit replay records and aggregate counts

Replay success does **not** imply profitability or live readiness.

