# Readiness Gate Hardening

Sprint 12 keeps expansion blocked by default and hardens the evidence gates around that decision.

## Default stance

`NeedMoreExperiments` remains the default outcome.

## Hard gates

- minimum usable datasets
- minimum outcome count
- minimum regime coverage
- minimum passed runs
- minimum average data quality
- no material regression vs previous campaign
- Risk Governor not blocking nearly everything
- Risk Governor not allowing too much without defensive value
- stable feature schema and clean leakage state

## Decision meanings

- `NeedMoreExperiments`: evidence still thin
- `ImproveDataFirst`: quality gate failed
- `ImproveRiskGovernorFirst`: risk behavior unstable
- `ImproveSignalModelFirst`: signal metrics too weak
- `HoldCurrentScope`: persona redundancy still too high
- `RegressedSinceLastCampaign`: previous evidence was better
- `ExpandToSixPersonas`: only when all hard gates pass and config explicitly allows recommendation
