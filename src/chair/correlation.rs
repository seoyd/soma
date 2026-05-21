pub fn cluster_multiplier(position_in_cluster: usize) -> f64 {
    match position_in_cluster {
        0 | 1 => 1.0,
        2 => 0.6,
        _ => 0.3,
    }
}
