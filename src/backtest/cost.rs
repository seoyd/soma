use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostModel {
    pub fee_bps: f64,
    pub slippage_bps: f64,
    pub spread_bps: Option<f64>,
    pub min_cost_bps: Option<f64>,
}

impl CostModel {
    pub fn estimate_round_trip_cost_bps(&self) -> f64 {
        let estimated = self.fee_bps.max(0.0) * 2.0
            + self.slippage_bps.max(0.0) * 2.0
            + self.spread_bps.unwrap_or(0.0).max(0.0);
        estimated.max(self.min_cost_bps.unwrap_or(0.0).max(0.0))
    }

    pub fn estimate_round_trip_cost_pct(&self) -> f64 {
        self.estimate_round_trip_cost_bps() / 10_000.0
    }

    pub fn net_return_after_cost(&self, gross_return_pct: f64) -> f64 {
        gross_return_pct - self.estimate_round_trip_cost_pct()
    }

    pub fn expected_edge_after_cost(&self, gross_edge_pct: f64) -> f64 {
        gross_edge_pct - self.estimate_round_trip_cost_pct()
    }
}
