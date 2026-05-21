fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

pub fn update_voice_power(current: f64, normalized_survival_score: f64, severe_event: bool) -> f64 {
    let mut next = 0.92 * clamp01(current) + 0.08 * clamp01(normalized_survival_score);
    if severe_event {
        next *= 0.5;
    }
    clamp01(next)
}
