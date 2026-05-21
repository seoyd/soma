# Promotion and Relegation

## Survival score

`survival_score` is a deterministic weighted score:

```text
0.22 * drawdown_control
+ 0.18 * risk_efficiency
+ 0.17 * net_expectancy_after_cost
+ 0.15 * calibration
+ 0.12 * regime_fit
+ 0.10 * silence_value
+ 0.06 * doctrine_consistency
- overconfidence_penalty
- overtrade_penalty
- correlation_penalty
- doctrine_violation_penalty
```

The final value is clamped to `[0, 1]`.

## Voice power update

Voice power uses a bounded EMA:

```text
next = 0.92 * current_voice_power + 0.08 * normalized_survival_score
```

On a severe event, the evaluation path halves the post-EMA value before clamping.

## Tier thresholds

Tier proposals derived from voice power are:

| Voice power | Proposed tier |
| --- | --- |
| `>= 0.80` | `S` |
| `>= 0.60` | `A` |
| `>= 0.40` | `B` |
| `>= 0.20` | `C` |
| `< 0.20` | `D` |

`S` is capped by `EvaluationProfile.max_s_tier`.

## Promotion rules

Promotion is allowed only when:

- `sample_count >= promotion_min_samples`
- `survival_score >= 0.60`
- `doctrine_violation_count == 0`
- `consecutive_bad_periods == 0`

If voice power suggests a higher tier but sample count is still too small, the persona stays in place and receives `PromotionInsufficientSamples`.

## Faster demotion

Demotion is intentionally easier than promotion. A persona is demoted when any of these fire:

- `high_confidence_miss_count >= 3`
- `consecutive_bad_periods >= 2`
- `survival_score < 0.25`
- domain mismatch plus very weak survival score

Severe events add `SevereDemotion`.

## Quarantine

`XQuarantined` is entered immediately when:

- `risk_bypass_attempt == true`, or
- a severe event occurs alongside doctrine violations

This is outside the normal ladder and requires explicit review before re-entry.

## NoTrade scoring

NoTrade is scored as a real outcome:

- avoided stop first: positive `silence_value`
- missed take-profit first: only a small negative penalty
- neutral non-trade: zero

This keeps defensive silence from looking like failure by default.

## Horizon and regime isolation

Evaluation is style-adjusted:

- matching horizon gets full score
- off-horizon still scores, but weaker
- favored regimes score highest
- tolerated regimes score lower
- out-of-domain regimes are penalized but not forced to zero

## Shadow evaluation

`ShadowVoteRecord` keeps future attribution hooks without touching the live path:

- `persona_id`
- `selected_for_decision`
- `affected_live_decision`
- `hypothetical_stance`
- `evaluation_pending`
