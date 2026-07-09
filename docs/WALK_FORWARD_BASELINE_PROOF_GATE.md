# Walk-Forward Baseline Proof Gate

## Why This Exists

Synthetic fixture success proves parser and report consistency. It does not
prove market edge. The proof gate asks a narrower and harder question: on
owner-provided sanitized local historical daily CSV data, does the committee
beat trivial baselines out of sample?

This is paper-only evaluation. It is not a profitability claim and not live
trading readiness.

## Four Baselines

`AlwaysNoTrade` holds cash and is the null strategy. If the committee does not
beat it, no edge is shown.

`BuyAndHold` buys at the first evaluation price and holds through the
evaluation window. It is the passive benchmark.

`EqualWeightCommittee` uses the three active agents with equal voice weight.
It removes adaptive voice advantage.

`VoiceAdaptiveCommittee` uses the existing voice/reward/cooldown feedback path
after paper outcomes. It must beat equal weight before voice adaptation is
trusted.

## Walk-Forward Rule

Splits use an expanding train window and fixed evaluation window. Evaluation
rows are out of sample. A decision at row `i` may use data available through
row `i`; the directional outcome is `close[i + 1] > close[i]`. The evaluator
does not use future rows to build the decision signal.

Training updates are intentionally minimal in this sprint. Fixed initial
states are used for equal weight. Voice-adaptive state may update only through
the existing paper feedback path.

## Metrics

Each baseline records:

- total return,
- max drawdown,
- trade, win, loss, no-trade, and risk-denial counts,
- average return per trade,
- optional volatility and Sharpe-like score when sample count is sufficient,
- downside loss,
- cost and slippage paid,
- reason codes.

Prediction quality records Brier score for directional probabilities,
calibrated sample count, missing probability count, abstention count,
high-confidence errors, low-confidence correct calls, mean confidence, and
mean realized direction.

## Proof And Failure Criteria

Progress means all comparisons are computed honestly from input data:

- VoiceAdaptiveCommittee beats EqualWeightCommittee,
- committee beats AlwaysNoTrade,
- committee beats BuyAndHold on a risk-adjusted score,
- prediction samples are sufficient.

Failure is explicit:

- VoiceAdaptiveCommittee does not beat EqualWeightCommittee,
- committee does not beat AlwaysNoTrade,
- committee does not beat BuyAndHold,
- insufficient sample count,
- no edge proven.

The report must keep these warnings visible:

- Local historical daily CSV only.
- Paper-only evaluation.
- No live trading readiness.
- No profitability claim.
- Voice adaptation must beat equal weight before it is trusted.
- Synthetic fixture success is not market evidence.
