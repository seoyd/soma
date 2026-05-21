pub mod audit;
pub mod contracts;
pub mod core_latency_budget;
pub mod determinism;
pub mod live_safety;
pub mod performance_budget;
pub mod readiness;
pub mod reason;
pub mod reason_audit;
pub mod runtime;
pub mod types;

pub use audit::{build_audit_event, stable_hash};
pub use contracts::{
    ContractCheckResult, ContractVersion, CoreContractRegistry, CoreContractRegistryReport,
};
pub use core_latency_budget::{
    CoreLatencyBudgetConfig, CoreLatencyBudgetReport, CoreLatencyBudgetStatus,
    build_core_latency_budget_report,
};
pub use determinism::{
    DeterminismCheck, DeterminismInputFingerprint, DeterminismOutputFingerprint,
    deterministic_float_format, stable_hash_string, stable_ordered_strings, stable_reason_codes,
};
pub use live_safety::{LiveSafetyReport, LiveSafetyStatus, build_live_safety_report};
pub use performance_budget::{
    ArtifactSize, CorePerformanceBudget, CorePerformanceBudgetReport, measure_performance_budget,
};
pub use readiness::{
    CoreCheckConfig, CoreCheckRunner, CoreNextRecommendation, CoreReadinessReport,
    CoreReadinessStatus, evaluate_core_readiness,
};
pub use reason::ReasonCode;
pub use reason_audit::{
    ReasonCodeAudit, ReasonCodeCompletenessStatus, audit_reason_codes, critical_reason_codes,
};
pub use runtime::{RuntimeMode, RuntimeStage, RuntimeState, RuntimeStateReport, RuntimeTransition};
pub use types::{
    AuditEvent, AuditEventType, ChairDecisionKind, ChairInput, ChairOutput, FeatureVector,
    InvestorVote, MarketSnapshot, OrderPlan, PaperOrder, PaperOrderStatus, PersonaTier, Regime,
    RiskDecision, RiskDecisionKind, RiskSnapshot, Side, SignalOutput, SixPrinciples, Stance,
    TradeProposal,
};
