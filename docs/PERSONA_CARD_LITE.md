# Persona card lite

`PersonaCardLite` is the bounded Sprint 32 schema for committee personas.

## Fields

- `immutable_doctrine`: hard or soft rules that shape veto/no-trade behavior
- `mutable_policy`: bounded thresholds such as confidence, spread, and volatility limits
- `horizon`: the intended decision horizon for the persona
- `role`: signal trigger, risk skeptic, regime guard, and related committee roles
- `source_compatibility`: which evidence sources the persona can speak on

## Vote output

Each scorer emits a `PersonaVote`:

- bounded `conviction`
- bounded `voice_power`
- source/horizon-aware `stance`
- doctrine violations and reason codes

Hard doctrine violations force a conservative result such as `Veto` or `NoTrade`.

