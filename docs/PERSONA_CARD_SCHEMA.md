# Persona Card Schema

## Core struct

`PersonaCard` is the stable contract for an active delegate.

| Field | Type | Meaning |
| --- | --- | --- |
| `persona_id` | `String` | Stable identifier used by Chair and tests |
| `archetype` | `String` | Human-readable strategy lineage |
| `tier` | `PersonaTier` | Current league tier |
| `immutable_doctrine` | `ImmutableDoctrine` | Non-negotiable strategy rules |
| `mutable_policy` | `MutablePolicy` | Tunable numeric policy knobs |
| `voice` | `VoiceConfig` | Current voice power and EMA controls |
| `evaluation` | `EvaluationProfile` | Horizon/regime domain and promotion policy |

## ImmutableDoctrine

Immutable doctrine carries rules that are not learned online and are not softened by Chair:

- `never_average_down`
- `cut_losses_quickly`
- `pyramid_only_on_strength`
- `speak_only_on_trend_or_breakout`
- `rest_after_consecutive_losses`
- `no_leverage`
- `do_not_speak_intraday_as_entry_signal`
- `reject_unknown_or_unscorable_asset`
- `margin_of_safety_required_when_fundamentals_available`
- `risk_first`
- `reject_poor_risk_reward`
- `reject_euphoria_chasing`
- `respect_cooldown`
- `no_trade_is_valid`

## MutablePolicy

Mutable policy holds numeric knobs that can be updated offline without changing doctrine:

- `breakout_lookback`
- `volume_z_threshold`
- `stop_loss_atr_mult`
- `take_profit_rr`
- `confidence_entry_threshold`
- `max_trade_frequency`
- `max_exposure_hint`
- `unknown_asset_penalty`
- `quality_threshold_placeholder`
- `defensive_bias`
- `overheat_threshold`
- `min_risk_reward`
- `volatility_penalty`
- `groupthink_penalty`
- `veto_sensitivity`

## EvaluationProfile

`EvaluationProfile` defines where a persona is allowed to be judged as in-domain:

| Field | Meaning |
| --- | --- |
| `horizon` | Primary horizon: `Intraday`, `Swing`, or `Position` |
| `favored_regimes` | Best-fit regimes |
| `tolerated_regimes` | Acceptable but weaker regimes |
| `promotion_min_samples` | Minimum observations before promotion |
| `max_s_tier` | Maximum simultaneous `S` seats allowed for that persona class |

## VoiceConfig

| Field | Meaning |
| --- | --- |
| `base_voice_power` | Starting influence |
| `current_voice_power` | Current influence used by Chair |
| `ema_alpha` | EMA smoothing factor |
| `severe_event_multiplier` | Additional downweighting on severe events |

## PersonaTier

`PersonaTier` is ordered as:

`XQuarantined < D < C < B < A < S`

`XQuarantined` removes the persona from normal promotion flow until explicitly reviewed.

## DoctrineViolation

The current violation set is:

- `AveragingDown`
- `IntradayEntrySignal`
- `UnknownAsset`
- `MarginOfSafetyMissing`
- `PoorRiskReward`
- `EuphoriaChasing`
- `CooldownIgnored`
- `RiskBypassAttempt`
