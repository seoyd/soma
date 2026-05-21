# Committee V1 operational MVP

Committee V1 means the 3-persona committee can now be run as a **single deterministic research bundle**:

1. load scenarios
2. replay persona/chair/risk decisions
3. score decision quality
4. generate chair/risk calibration suggestions
5. evaluate Committee V1 readiness
6. write a local report bundle

## What it means

- the committee path is operational as a research MVP
- outputs are deterministic and file-based
- scenario loading, replay, diagnostics, calibration, and readiness now live on one path

## What it does not mean

- no live trading
- no broker/order/account APIs
- no runtime LLM
- no Mamba runtime
- no real-money recommendation
- no claim of reproducing real investors

## Evidence flow

`CommitteeScenarioSet -> CommitteeReplayReport -> diagnostics/quality/calibration/readiness -> CommitteeV1ReportBundle`

YFinance remains research-only. Fixture rows remain architecture-test-only.

