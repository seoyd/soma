# Restart Sprint 34 Report

## Outcome

The Sprint ended in `RealSmokeFailed`. The authorized public Upbit daily-candle request reached canonical parsing and returned a bounded 200-row `KRW-BTC` page, but local snapshot reload verification rejected it for a digest mismatch. No rejected local artifact is treated as historical evidence.

## Changes

* Local snapshot writes now verify the input, the temporary file, and the renamed file before reporting success.
* Daily replay datasets use one bit-stable canonical digest contract across broker acquisition, Upbit storage, and historical-evidence inventory.
* The local-only campaign command can load exactly one existing snapshot, calculate sufficiency, freeze and verify evidence, then invoke the existing ShadowOnly campaign only after inventory acceptance.
* Frozen encoder construction derives its input width from the existing feature schema and uses the configured campaign seed and backend policy.

## Execution Result

* Provider scope remained public, read-only Upbit daily OHLCV over HTTPS GET.
* Each authorized attempt was sequential and bounded to one configured page; no provider was added and no account, order, streaming, or background path was used.
* The actual page had 200 normalized rows, which meets the existing campaign row/window calculation, but it was not accepted because snapshot digest reload verification failed.
* No accepted final snapshot, frozen real evidence pack, campaign result, Mamba/Linear comparison, drift result, or model verdict exists.
* No provider call was made after the acquisition phase was closed.

## Verification

* Default workspace tests: 598 passed.
* `backend-metal` workspace tests: 599 passed.
* Focused Upbit storage tests: 7 passed.
* Focused historical-evidence tests: 9 passed.

Known unrelated warnings remain the two existing unused functions in `persona_card.rs`.

## Boundaries

All models remain `ShadowOnly`; active committee membership, Chair, Risk Governor, PaperBroker, execution authority, and official Mamba conformance remain unchanged. Local configuration and real/failed snapshot artifacts are ignored and were not committed.
