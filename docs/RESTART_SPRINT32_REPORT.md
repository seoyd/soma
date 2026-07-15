# Restart Sprint 32 Report

## Verification

Baseline verification completed sequentially: formatting, workspace check, default workspace tests, Metal workspace check, and Metal workspace tests. Provider-specific unit tests remain network-free.

## Provider Decision

Upbit was selected as the one pilot candidate because the repository already contained a public daily-candle collector profile and Upbit’s official documentation specifies the unauthenticated quotation operation, daily OHLCV schema, UTC candle semantics, `market`/`to`/`count` request contract, 200-candle request cap, and candle rate limit. Other providers were not activated or given fallback behavior.

## Implementation

The pilot adds computed qualification and selection, default-denied manual network consent, a fixed HTTPS GET endpoint allowlist, bounded timeout/retry/response-size configuration, canonical parser validation, broker-backed immutable snapshot creation, atomic ignored local storage, and re-read digest verification. The command remains manual and does not run automatically in tests, CI, or application startup.

## Execution Result

No real smoke result is claimed unless a local operator creates the ignored configuration, enables manual consent, and runs the explicit command. No real snapshot or Momentum campaign is claimed by this report.

## Boundaries

No credential, account, order, cancel, transfer, WebSocket, background polling, trading, model promotion, GPU training, or runtime LLM path was added. A pilot snapshot is not enough to relax the existing historical evidence or campaign thresholds.

## Next Step

If an operator elects to run the manual smoke, re-run the existing snapshot inventory on the verified local snapshot. Run the existing per-series campaign only after the configured real-evidence thresholds are met.
