pub mod attribution;
pub mod candle;
pub mod cost;
pub mod ledger;
pub mod simulator;
pub mod triple_barrier;

pub use attribution::{AttributionRecord, CounterfactualRole, ShadowOutcomeRecord};
pub use candle::{Candle, CandleSeries, MarketReplayCursor, Timeframe};
pub use cost::CostModel;
pub use ledger::{DecisionRecord, NoTradeEvaluation, OutcomeRecord};
pub use simulator::{
    BacktestConfig, BacktestResult, BacktestSimulator, NoTradeScoreConfig, SimulationResult,
    evaluate_no_trade_counterfactual, simulate_paper_cycle,
};
pub use triple_barrier::{
    BarrierHit, TripleBarrierConfig, TripleBarrierOutcome, TripleBarrierResult,
    evaluate_triple_barrier,
};
