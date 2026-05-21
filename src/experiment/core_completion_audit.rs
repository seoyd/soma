use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};

fn default_output_root() -> String {
    "target/sprint55/core_completion_audit".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoreSubsystem {
    RuntimeStateMachine,
    ContractRegistry,
    DeterminismGuard,
    ReasonCodeAudit,
    AuditLedger,
    RiskInvariants,
    LiveSafety,
    PerformanceBudget,
    CoreReadiness,
    CoreCheckedBenchmark,
    CorePerformanceScorecard,
    ProviderPipeline,
    KISMarketDataPath,
    EvidenceMaterialization,
    OutcomeLinkage,
    CounterfactualDepth,
    CommitteeTrinity,
    ChairV0,
    RiskGovernor,
    OwnerInputLayer,
    ControlTowerV1,
    LiveTradingSurface,
    KISOrderAccountSurface,
    Mamba3Runtime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SubsystemMaturity {
    Missing,
    Prototype,
    ResearchReady,
    PaperReady,
    Blocked,
    Deferred,
    Forbidden,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreSubsystemMaturityRow {
    pub subsystem: CoreSubsystem,
    pub maturity: SubsystemMaturity,
    pub evidence_summary: String,
    #[serde(default)]
    pub blockers: Vec<String>,
    pub next_action: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreSubsystemMaturityMatrix {
    pub rows: Vec<CoreSubsystemMaturityRow>,
    pub research_ready_count: usize,
    pub paper_ready_count: usize,
    pub blocked_count: usize,
    pub deferred_count: usize,
    pub forbidden_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CoreSubsystemMaturityMatrix {
    pub fn to_text(&self) -> String {
        [
            format!("research_ready_count={}", self.research_ready_count),
            format!("paper_ready_count={}", self.paper_ready_count),
            format!("blocked_count={}", self.blocked_count),
            format!("deferred_count={}", self.deferred_count),
            format!("forbidden_count={}", self.forbidden_count),
            format!(
                "rows={}",
                self.rows
                    .iter()
                    .map(|row| format!("{:?}:{:?}", row.subsystem, row.maturity))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreCompletionStatus {
    CoreResearchOperatingSystemComplete,
    CorePaperOperatingSystemComplete,
    CoreNeedsEvidenceDepth,
    CoreNeedsRiskReview,
    CoreNeedsControlTowerWork,
    CoreBlockedByLiveSafety,
    CoreBlockedByDeterminism,
    CoreBlockedByEvidence,
    CoreIncomplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreCompletionRecommendation {
    CoreResearchOperatingSystemComplete,
    CoreNeedsKISEvidenceDepth,
    CoreNeedsOutcomeLinkDepth,
    CoreNeedsRiskReview,
    CoreNeedsControlTowerWork,
    HoldModelComplexity,
    BuildSequenceDatasetFirst,
    KeepTrinity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreRemainingGap {
    pub gap_id: String,
    pub subsystem: Option<CoreSubsystem>,
    pub summary: String,
    pub next_action: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreRemainingGapReport {
    pub audit_id: String,
    pub gaps: Vec<CoreRemainingGap>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CoreRemainingGapReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("audit_id={}", self.audit_id),
            format!(
                "gaps={}",
                self.gaps
                    .iter()
                    .map(|gap| format!("{}:{}", gap.gap_id, gap.next_action))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreCompletionAuditReport {
    pub audit_id: String,
    pub maturity_matrix: CoreSubsystemMaturityMatrix,
    pub passed_core_requirements: Vec<String>,
    pub failed_core_requirements: Vec<String>,
    pub missing_subsystems: Vec<CoreSubsystem>,
    pub blocked_subsystems: Vec<CoreSubsystem>,
    pub deferred_subsystems: Vec<CoreSubsystem>,
    pub forbidden_subsystems: Vec<CoreSubsystem>,
    pub core_completion_status: CoreCompletionStatus,
    pub final_recommendation: CoreCompletionRecommendation,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CoreCompletionAuditReport {
    pub fn from_value(value: &Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok().or_else(|| {
            value
                .get("core_completion_audit_report")
                .and_then(|item| serde_json::from_value(item.clone()).ok())
        })
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("audit_id={}", self.audit_id),
            format!("core_completion_status={:?}", self.core_completion_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!(
                "passed_core_requirements={}",
                self.passed_core_requirements.join("|")
            ),
            format!(
                "failed_core_requirements={}",
                self.failed_core_requirements.join("|")
            ),
            format!(
                "blocked_subsystems={}",
                self.blocked_subsystems
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!(
                "deferred_subsystems={}",
                self.deferred_subsystems
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!(
                "forbidden_subsystems={}",
                self.forbidden_subsystems
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            self.maturity_matrix.to_text(),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreCompletionAuditConfig {
    pub audit_id: String,
    #[serde(default)]
    pub core_check_report_paths: Vec<String>,
    #[serde(default)]
    pub core_performance_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_state_paths: Vec<String>,
    #[serde(default)]
    pub kis_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_collection_closure_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_scaleout_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_diversity_paths: Vec<String>,
    #[serde(default)]
    pub committee_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub owner_impact_report_paths: Vec<String>,
    #[serde(default)]
    pub risk_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_core_check_pass: bool,
    #[serde(default = "default_true")]
    pub require_risk_invariants: bool,
    #[serde(default = "default_true")]
    pub require_live_safety_pass: bool,
    #[serde(default = "default_true")]
    pub require_control_tower_ready: bool,
    #[serde(default = "default_true")]
    pub require_owner_layer_ready: bool,
    #[serde(default = "default_true")]
    pub require_kis_market_data_ready: bool,
    #[serde(default = "default_true")]
    pub require_no_live_paths: bool,
    #[serde(default = "default_true")]
    pub require_no_broker_paths: bool,
    #[serde(default = "default_true")]
    pub require_no_mamba_runtime: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CoreCompletionAuditConfig {
    fn default() -> Self {
        Self {
            audit_id: "sprint55_core_completion".to_string(),
            core_check_report_paths: Vec::new(),
            core_performance_scorecard_paths: Vec::new(),
            control_tower_state_paths: Vec::new(),
            kis_activation_report_paths: Vec::new(),
            kis_collection_closure_paths: Vec::new(),
            official_evidence_scaleout_paths: Vec::new(),
            official_evidence_diversity_paths: Vec::new(),
            committee_benchmark_paths: Vec::new(),
            owner_impact_report_paths: Vec::new(),
            risk_report_paths: Vec::new(),
            output_root: default_output_root(),
            require_core_check_pass: true,
            require_risk_invariants: true,
            require_live_safety_pass: true,
            require_control_tower_ready: true,
            require_owner_layer_ready: true,
            require_kis_market_data_ready: true,
            require_no_live_paths: true,
            require_no_broker_paths: true,
            require_no_mamba_runtime: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CoreCompletionAuditConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        if self
            .all_input_paths()
            .iter()
            .chain([self.output_root.clone()].iter())
            .any(|path| path.contains("://"))
        {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::RemotePathRejected,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.audit_id.trim().is_empty() {
            return Err("core completion audit id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("core-completion-audit config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.audit_id)
    }

    pub fn all_input_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .core_check_report_paths
                .iter()
                .chain(self.core_performance_scorecard_paths.iter())
                .chain(self.control_tower_state_paths.iter())
                .chain(self.kis_activation_report_paths.iter())
                .chain(self.kis_collection_closure_paths.iter())
                .chain(self.official_evidence_scaleout_paths.iter())
                .chain(self.official_evidence_diversity_paths.iter())
                .chain(self.committee_benchmark_paths.iter())
                .chain(self.owner_impact_report_paths.iter())
                .chain(self.risk_report_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoreCompletionAuditRunner;

impl CoreCompletionAuditRunner {
    pub fn run(
        &self,
        config: &CoreCompletionAuditConfig,
    ) -> Result<(CoreCompletionAuditReport, CoreRemainingGapReport), String> {
        config.validate()?;
        let mut warnings = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        let values = load_values(&config.all_input_paths(), &mut warnings, &mut reason_codes);
        let snapshot = AuditSnapshot::from_values(&values);

        let rows = vec![
            maturity_row(
                CoreSubsystem::RuntimeStateMachine,
                if snapshot.runtime_state_machine_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Missing
                },
                "runtime state machine remains deterministic and research-only",
                if snapshot.runtime_state_machine_ready {
                    Vec::new()
                } else {
                    vec!["runtime state machine evidence missing".to_string()]
                },
                "keep deterministic runtime transitions and no live trading path",
            ),
            maturity_row(
                CoreSubsystem::ContractRegistry,
                if snapshot.contract_registry_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Missing
                },
                "contract registry exists for stable research interfaces",
                if snapshot.contract_registry_ready {
                    Vec::new()
                } else {
                    vec!["contract registry evidence missing".to_string()]
                },
                "keep contract boundaries frozen before new scope",
            ),
            maturity_row(
                CoreSubsystem::DeterminismGuard,
                if snapshot.determinism_guard_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Blocked
                },
                "determinism guard exists for local-first repeatable outputs",
                if snapshot.determinism_guard_ready {
                    Vec::new()
                } else {
                    vec!["determinism guard evidence missing or failing".to_string()]
                },
                "fix determinism before adding model complexity",
            ),
            maturity_row(
                CoreSubsystem::ReasonCodeAudit,
                if snapshot.reason_code_audit_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "reason-code audit exists for bounded numeric decisions",
                if snapshot.reason_code_audit_ready {
                    Vec::new()
                } else {
                    vec!["reason-code audit evidence is still thin".to_string()]
                },
                "expand reason-code coverage before stronger claims",
            ),
            maturity_row(
                CoreSubsystem::AuditLedger,
                if snapshot.audit_ledger_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "audit ledger exists for paper-only accountability",
                if snapshot.audit_ledger_ready {
                    Vec::new()
                } else {
                    vec!["audit ledger evidence missing".to_string()]
                },
                "keep audit ledger complete and local-only",
            ),
            maturity_row(
                CoreSubsystem::RiskInvariants,
                if snapshot.risk_invariants_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Blocked
                },
                "risk invariants remain absolute on the decision path",
                if snapshot.risk_invariants_ready {
                    Vec::new()
                } else {
                    vec!["risk invariants are not yet proven".to_string()]
                },
                "restore risk invariants before changing committee or models",
            ),
            maturity_row(
                CoreSubsystem::LiveSafety,
                if snapshot.live_safety_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Blocked
                },
                "live safety proof exists, but core completion still does not imply live readiness",
                if snapshot.live_safety_ready {
                    Vec::new()
                } else {
                    vec!["live safety proof missing or incomplete".to_string()]
                },
                "keep paper-only boundaries and no live execution",
            ),
            maturity_row(
                CoreSubsystem::PerformanceBudget,
                if snapshot.performance_budget_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "performance budget exists to limit deterministic runtime cost",
                if snapshot.performance_budget_ready {
                    Vec::new()
                } else {
                    vec!["performance budget evidence missing".to_string()]
                },
                "measure cost before heavier sequence work",
            ),
            maturity_row(
                CoreSubsystem::CoreReadiness,
                if snapshot.core_readiness_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Blocked
                },
                "core readiness report exists for research OS interpretation",
                if snapshot.core_readiness_ready {
                    Vec::new()
                } else {
                    vec!["core readiness evidence missing".to_string()]
                },
                "rebuild readiness evidence before escalation",
            ),
            maturity_row(
                CoreSubsystem::CoreCheckedBenchmark,
                if snapshot.core_checked_benchmark_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "core-check benchmark gate exists for paper-only evaluation",
                if snapshot.core_checked_benchmark_ready {
                    Vec::new()
                } else {
                    vec!["core-checked benchmark evidence missing".to_string()]
                },
                "keep core-check gating benchmark execution",
            ),
            maturity_row(
                CoreSubsystem::CorePerformanceScorecard,
                if snapshot.core_performance_scorecard_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "core performance scorecard exists but does not imply profitability",
                if snapshot.core_performance_scorecard_ready {
                    Vec::new()
                } else {
                    vec!["core performance scorecard evidence missing".to_string()]
                },
                "keep scorecards diagnostic and cost-aware",
            ),
            maturity_row(
                CoreSubsystem::ProviderPipeline,
                if snapshot.provider_pipeline_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "provider pipeline is bounded and market-data-only",
                if snapshot.provider_pipeline_ready {
                    Vec::new()
                } else {
                    vec!["provider pipeline evidence missing".to_string()]
                },
                "keep provider scope bounded and official-first",
            ),
            maturity_row(
                CoreSubsystem::KISMarketDataPath,
                if snapshot.kis_market_data_ready {
                    SubsystemMaturity::PaperReady
                } else {
                    SubsystemMaturity::Blocked
                },
                "KIS remains market-data-only and bounded for paper workflows",
                if snapshot.kis_market_data_ready {
                    Vec::new()
                } else {
                    vec!["KIS market-data readiness evidence missing".to_string()]
                },
                "improve official KIS market-data evidence depth",
            ),
            maturity_row(
                CoreSubsystem::EvidenceMaterialization,
                if snapshot.official_row_count > 0 {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Blocked
                },
                &format!(
                    "official evidence rows={}, complete_rows={}",
                    snapshot.official_row_count, snapshot.complete_row_count
                ),
                if snapshot.official_row_count > 0 {
                    Vec::new()
                } else {
                    vec!["official evidence rows are still missing".to_string()]
                },
                "materialize more official rows with provenance before stronger claims",
            ),
            maturity_row(
                CoreSubsystem::OutcomeLinkage,
                if snapshot.outcome_link_depth_weak {
                    SubsystemMaturity::Blocked
                } else if snapshot.outcome_links > 0 {
                    SubsystemMaturity::PaperReady
                } else {
                    SubsystemMaturity::Prototype
                },
                &format!("outcome_links={}", snapshot.outcome_links),
                if snapshot.outcome_link_depth_weak {
                    vec![
                        "outcome-link depth remains weak for conservative interpretation"
                            .to_string(),
                    ]
                } else {
                    Vec::new()
                },
                "expand official outcome linkage before escalating models",
            ),
            maturity_row(
                CoreSubsystem::CounterfactualDepth,
                if snapshot.counterfactual_depth_weak {
                    SubsystemMaturity::Blocked
                } else if snapshot.counterfactuals > 0 {
                    SubsystemMaturity::PaperReady
                } else {
                    SubsystemMaturity::Prototype
                },
                &format!("counterfactuals={}", snapshot.counterfactuals),
                if snapshot.counterfactual_depth_weak {
                    vec![
                        "counterfactual depth remains weak for stronger performance interpretation"
                            .to_string(),
                    ]
                } else {
                    Vec::new()
                },
                "grow no-trade and risk-denied counterfactual depth",
            ),
            maturity_row(
                CoreSubsystem::CommitteeTrinity,
                if snapshot.committee_trinity_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "minimal trinity committee remains the bounded active committee",
                if snapshot.committee_trinity_ready {
                    Vec::new()
                } else {
                    vec!["committee trinity evidence missing".to_string()]
                },
                "keep trinity and avoid 6/12/18 active expansion",
            ),
            maturity_row(
                CoreSubsystem::ChairV0,
                if snapshot.chair_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "ChairV0 exists but cannot bypass Risk Governor",
                if snapshot.chair_ready {
                    Vec::new()
                } else {
                    vec!["chair evidence missing".to_string()]
                },
                "tune Chair only after evidence depth improves",
            ),
            maturity_row(
                CoreSubsystem::RiskGovernor,
                if snapshot.risk_governor_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Blocked
                },
                "Risk Governor remains absolute veto and NoTrade default",
                if snapshot.risk_governor_ready {
                    Vec::new()
                } else {
                    vec!["risk governor evidence missing".to_string()]
                },
                "keep veto absolute before any escalation",
            ),
            maturity_row(
                CoreSubsystem::OwnerInputLayer,
                if snapshot.owner_input_ready {
                    SubsystemMaturity::ResearchReady
                } else {
                    SubsystemMaturity::Prototype
                },
                "owner input layer exists but remains unable to bypass risk",
                if snapshot.owner_input_ready {
                    Vec::new()
                } else {
                    vec!["owner input readiness evidence missing".to_string()]
                },
                "keep owner drafts paper-only and audit-backed",
            ),
            maturity_row(
                CoreSubsystem::ControlTowerV1,
                if snapshot.control_tower_ready {
                    SubsystemMaturity::PaperReady
                } else {
                    SubsystemMaturity::Blocked
                },
                "Control Tower v1 is read-only, local-only, and paper-only",
                if snapshot.control_tower_ready {
                    Vec::new()
                } else {
                    vec!["Control Tower v1 readiness evidence missing".to_string()]
                },
                "keep Control Tower read-only and local-first",
            ),
            maturity_row(
                CoreSubsystem::LiveTradingSurface,
                SubsystemMaturity::Forbidden,
                "live trading surface remains forbidden in Sprint 55",
                Vec::new(),
                "keep live trading forbidden",
            ),
            maturity_row(
                CoreSubsystem::KISOrderAccountSurface,
                SubsystemMaturity::Forbidden,
                "KIS order/account/balance/holdings/position endpoints remain forbidden",
                Vec::new(),
                "keep KIS market-data-only",
            ),
            maturity_row(
                CoreSubsystem::Mamba3Runtime,
                if snapshot.mamba_runtime_present {
                    SubsystemMaturity::Forbidden
                } else {
                    SubsystemMaturity::Deferred
                },
                if snapshot.mamba_runtime_present {
                    "Mamba3 runtime is not allowed in Sprint 55"
                } else {
                    "Mamba3 runtime remains deferred and unimplemented"
                },
                if snapshot.mamba_runtime_present {
                    vec!["Mamba3 runtime presence would violate Sprint 55 scope".to_string()]
                } else {
                    Vec::new()
                },
                "keep runtime deferred; only research audit/gating is in scope",
            ),
        ];

        let matrix = build_matrix(rows, &reason_codes);
        let blocked_subsystems = rows_with(&matrix.rows, SubsystemMaturity::Blocked);
        let deferred_subsystems = rows_with(&matrix.rows, SubsystemMaturity::Deferred);
        let forbidden_subsystems = rows_with(&matrix.rows, SubsystemMaturity::Forbidden);
        let missing_subsystems = rows_with(&matrix.rows, SubsystemMaturity::Missing);

        let mut passed_core_requirements = Vec::new();
        let mut failed_core_requirements = Vec::new();
        requirement(
            config.require_core_check_pass,
            snapshot.core_readiness_ready,
            "CoreCheckPassed",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_risk_invariants,
            snapshot.risk_invariants_ready,
            "RiskInvariants",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_live_safety_pass,
            snapshot.live_safety_ready,
            "LiveSafety",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_control_tower_ready,
            snapshot.control_tower_ready,
            "ControlTowerReady",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_owner_layer_ready,
            snapshot.owner_input_ready,
            "OwnerLayerReady",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_kis_market_data_ready,
            snapshot.kis_market_data_ready,
            "KISMarketDataReady",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_no_live_paths,
            snapshot.no_live_paths,
            "NoLivePaths",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_no_broker_paths,
            snapshot.no_broker_paths,
            "NoBrokerPaths",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );
        requirement(
            config.require_no_mamba_runtime,
            !snapshot.mamba_runtime_present,
            "NoMambaRuntime",
            &mut passed_core_requirements,
            &mut failed_core_requirements,
        );

        let operational_core_ready = [
            snapshot.runtime_state_machine_ready,
            snapshot.contract_registry_ready,
            snapshot.determinism_guard_ready,
            snapshot.reason_code_audit_ready,
            snapshot.audit_ledger_ready,
            snapshot.risk_invariants_ready,
            snapshot.live_safety_ready,
            snapshot.performance_budget_ready,
            snapshot.core_readiness_ready,
            snapshot.core_checked_benchmark_ready,
            snapshot.core_performance_scorecard_ready,
            snapshot.provider_pipeline_ready,
            snapshot.kis_market_data_ready,
            snapshot.committee_trinity_ready,
            snapshot.chair_ready,
            snapshot.risk_governor_ready,
            snapshot.owner_input_ready,
            snapshot.control_tower_ready,
            snapshot.no_live_paths,
            snapshot.no_broker_paths,
            !snapshot.mamba_runtime_present,
        ]
        .into_iter()
        .all(|flag| flag);

        let core_completion_status = if !snapshot.determinism_guard_ready {
            CoreCompletionStatus::CoreBlockedByDeterminism
        } else if !snapshot.live_safety_ready {
            CoreCompletionStatus::CoreBlockedByLiveSafety
        } else if snapshot.risk_review_needed {
            CoreCompletionStatus::CoreNeedsRiskReview
        } else if snapshot.control_tower_work_needed {
            CoreCompletionStatus::CoreNeedsControlTowerWork
        } else if operational_core_ready {
            CoreCompletionStatus::CoreResearchOperatingSystemComplete
        } else if snapshot.kis_evidence_depth_weak || snapshot.outcome_link_depth_weak {
            CoreCompletionStatus::CoreNeedsEvidenceDepth
        } else {
            CoreCompletionStatus::CoreIncomplete
        };

        let final_recommendation = if snapshot.outcome_link_depth_weak {
            CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth
        } else if snapshot.kis_evidence_depth_weak {
            CoreCompletionRecommendation::CoreNeedsKISEvidenceDepth
        } else if snapshot.risk_review_needed {
            CoreCompletionRecommendation::CoreNeedsRiskReview
        } else if snapshot.control_tower_work_needed {
            CoreCompletionRecommendation::CoreNeedsControlTowerWork
        } else if matrix.blocked_count > 0 {
            CoreCompletionRecommendation::HoldModelComplexity
        } else {
            CoreCompletionRecommendation::KeepTrinity
        };

        warnings.push(
            "core completion does not imply live trading readiness or profitability".to_string(),
        );
        if snapshot.kis_evidence_depth_weak {
            warnings.push(
                "KIS evidence depth remains weak, so interpretation must stay conservative"
                    .to_string(),
            );
        }
        if snapshot.outcome_link_depth_weak {
            warnings.push(
                "official outcome-link depth remains weak, so model escalation stays conservative"
                    .to_string(),
            );
        }

        let report = CoreCompletionAuditReport {
            audit_id: config.audit_id.clone(),
            maturity_matrix: matrix,
            passed_core_requirements: stable_ordered_strings(&passed_core_requirements),
            failed_core_requirements: stable_ordered_strings(&failed_core_requirements),
            missing_subsystems,
            blocked_subsystems: blocked_subsystems.clone(),
            deferred_subsystems: deferred_subsystems.clone(),
            forbidden_subsystems: forbidden_subsystems.clone(),
            core_completion_status,
            final_recommendation,
            warnings: stable_ordered_strings(&warnings),
            reason_codes: stable_reason_codes(
                &[
                    reason_codes.clone(),
                    vec![
                        ReasonCode::CoreReadinessBuilt,
                        ReasonCode::DeterministicPath,
                    ],
                ]
                .concat(),
            ),
        };
        let gap_report =
            build_gap_report(config, &report, &blocked_subsystems, &deferred_subsystems);
        write_reports(&config.output_dir(), &report, &gap_report)?;
        Ok((report, gap_report))
    }
}

fn write_reports(
    output_dir: &Path,
    report: &CoreCompletionAuditReport,
    gap_report: &CoreRemainingGapReport,
) -> Result<(), String> {
    fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("core_completion_audit_report.json"),
        report.to_json_string()?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("core_completion_audit_report.txt"),
        report.to_text(),
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("core_subsystem_maturity_matrix.json"),
        serde_json::to_string_pretty(&report.maturity_matrix).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("core_subsystem_maturity_matrix.txt"),
        report.maturity_matrix.to_text(),
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("core_remaining_gap_report.json"),
        gap_report.to_json_string()?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("core_remaining_gap_report.txt"),
        gap_report.to_text(),
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn build_gap_report(
    config: &CoreCompletionAuditConfig,
    report: &CoreCompletionAuditReport,
    blocked_subsystems: &[CoreSubsystem],
    deferred_subsystems: &[CoreSubsystem],
) -> CoreRemainingGapReport {
    let mut gaps = Vec::new();
    for subsystem in blocked_subsystems {
        gaps.push(CoreRemainingGap {
            gap_id: format!("{:?}-blocked", subsystem),
            subsystem: Some(*subsystem),
            summary: format!("{:?} is still blocked", subsystem),
            next_action: format!("review {:?} before more model scope", subsystem),
        });
    }
    for subsystem in deferred_subsystems {
        gaps.push(CoreRemainingGap {
            gap_id: format!("{:?}-deferred", subsystem),
            subsystem: Some(*subsystem),
            summary: format!("{:?} remains deferred by policy", subsystem),
            next_action: format!("keep {:?} deferred in Sprint 55", subsystem),
        });
    }
    if matches!(
        report.final_recommendation,
        CoreCompletionRecommendation::CoreNeedsKISEvidenceDepth
    ) {
        gaps.push(CoreRemainingGap {
            gap_id: "kis-evidence-depth".to_string(),
            subsystem: Some(CoreSubsystem::KISMarketDataPath),
            summary: "KIS evidence depth is still too thin for stronger claims".to_string(),
            next_action: "expand official KIS evidence before escalating models".to_string(),
        });
    }
    if matches!(
        report.final_recommendation,
        CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth
    ) {
        gaps.push(CoreRemainingGap {
            gap_id: "outcome-link-depth".to_string(),
            subsystem: Some(CoreSubsystem::OutcomeLinkage),
            summary: "official outcome links and counterfactual depth remain thin".to_string(),
            next_action: "improve outcome-link depth and counterfactual coverage first".to_string(),
        });
    }
    CoreRemainingGapReport {
        audit_id: config.audit_id.clone(),
        gaps,
        warnings: report.warnings.clone(),
        reason_codes: stable_reason_codes(&[
            ReasonCode::CoreReadinessBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn build_matrix(
    rows: Vec<CoreSubsystemMaturityRow>,
    reason_codes: &[ReasonCode],
) -> CoreSubsystemMaturityMatrix {
    let research_ready_count = rows
        .iter()
        .filter(|row| matches!(row.maturity, SubsystemMaturity::ResearchReady))
        .count();
    let paper_ready_count = rows
        .iter()
        .filter(|row| matches!(row.maturity, SubsystemMaturity::PaperReady))
        .count();
    let blocked_count = rows
        .iter()
        .filter(|row| matches!(row.maturity, SubsystemMaturity::Blocked))
        .count();
    let deferred_count = rows
        .iter()
        .filter(|row| matches!(row.maturity, SubsystemMaturity::Deferred))
        .count();
    let forbidden_count = rows
        .iter()
        .filter(|row| matches!(row.maturity, SubsystemMaturity::Forbidden))
        .count();
    CoreSubsystemMaturityMatrix {
        rows,
        research_ready_count,
        paper_ready_count,
        blocked_count,
        deferred_count,
        forbidden_count,
        reason_codes: stable_reason_codes(reason_codes),
    }
}

fn rows_with(rows: &[CoreSubsystemMaturityRow], maturity: SubsystemMaturity) -> Vec<CoreSubsystem> {
    rows.iter()
        .filter(|row| row.maturity == maturity)
        .map(|row| row.subsystem)
        .collect()
}

fn maturity_row(
    subsystem: CoreSubsystem,
    maturity: SubsystemMaturity,
    evidence_summary: &str,
    blockers: Vec<String>,
    next_action: &str,
) -> CoreSubsystemMaturityRow {
    CoreSubsystemMaturityRow {
        subsystem,
        maturity,
        evidence_summary: evidence_summary.to_string(),
        blockers: stable_ordered_strings(&blockers),
        next_action: next_action.to_string(),
        reason_codes: stable_reason_codes(&[ReasonCode::DeterministicPath]),
    }
}

fn requirement(
    required: bool,
    passed: bool,
    label: &str,
    passed_output: &mut Vec<String>,
    failed_output: &mut Vec<String>,
) {
    if !required {
        return;
    }
    if passed {
        passed_output.push(label.to_string());
    } else {
        failed_output.push(label.to_string());
    }
}

fn load_values(
    paths: &[String],
    warnings: &mut Vec<String>,
    reason_codes: &mut Vec<ReasonCode>,
) -> Vec<Value> {
    let mut values = Vec::new();
    for path in stable_ordered_strings(paths) {
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => values.push(value),
                Err(err) => {
                    warnings.push(format!("failed to parse core audit input: {err}"));
                    reason_codes.push(ReasonCode::DataLoadFailed);
                }
            },
            Err(_) => {
                warnings.push(format!("missing core audit input: {path}"));
                reason_codes.push(ReasonCode::MissingFile);
            }
        }
    }
    values
}

#[derive(Clone, Debug, Default)]
struct AuditSnapshot {
    runtime_state_machine_ready: bool,
    contract_registry_ready: bool,
    determinism_guard_ready: bool,
    reason_code_audit_ready: bool,
    audit_ledger_ready: bool,
    risk_invariants_ready: bool,
    live_safety_ready: bool,
    performance_budget_ready: bool,
    core_readiness_ready: bool,
    core_checked_benchmark_ready: bool,
    core_performance_scorecard_ready: bool,
    provider_pipeline_ready: bool,
    kis_market_data_ready: bool,
    committee_trinity_ready: bool,
    chair_ready: bool,
    risk_governor_ready: bool,
    owner_input_ready: bool,
    control_tower_ready: bool,
    no_live_paths: bool,
    no_broker_paths: bool,
    mamba_runtime_present: bool,
    official_row_count: usize,
    complete_row_count: usize,
    outcome_links: usize,
    counterfactuals: usize,
    kis_evidence_depth_weak: bool,
    outcome_link_depth_weak: bool,
    counterfactual_depth_weak: bool,
    risk_review_needed: bool,
    control_tower_work_needed: bool,
}

impl AuditSnapshot {
    fn from_values(values: &[Value]) -> Self {
        let mut snapshot = AuditSnapshot {
            no_live_paths: true,
            no_broker_paths: true,
            ..AuditSnapshot::default()
        };
        for value in values {
            snapshot.runtime_state_machine_ready |=
                bool_field(value, &["runtime_state_machine_ready", "runtime_ready"])
                    .unwrap_or(false);
            snapshot.contract_registry_ready |=
                bool_field(value, &["contract_registry_ready", "contracts_ready"]).unwrap_or(false);
            snapshot.determinism_guard_ready |=
                bool_field(value, &["determinism_guard_ready", "determinism_ready"])
                    .unwrap_or(false);
            snapshot.reason_code_audit_ready |=
                bool_field(value, &["reason_code_audit_ready", "reason_codes_ready"])
                    .unwrap_or(false);
            snapshot.audit_ledger_ready |=
                bool_field(value, &["audit_ledger_ready", "audit_ready"]).unwrap_or(false);
            snapshot.risk_invariants_ready |=
                bool_field(value, &["risk_invariants_ready", "risk_ready"]).unwrap_or(false);
            snapshot.live_safety_ready |=
                bool_field(value, &["live_safety_ready", "live_safety_pass"]).unwrap_or(false);
            snapshot.performance_budget_ready |=
                bool_field(value, &["performance_budget_ready", "budget_ready"]).unwrap_or(false);
            snapshot.core_readiness_ready |= bool_field(
                value,
                &["core_readiness_ready", "core_check_passed", "core_ready"],
            )
            .unwrap_or(false);
            snapshot.core_checked_benchmark_ready |=
                bool_field(value, &["core_checked_benchmark_ready", "benchmark_ready"])
                    .unwrap_or(false);
            snapshot.core_performance_scorecard_ready |= bool_field(
                value,
                &["core_performance_scorecard_ready", "scorecard_ready"],
            )
            .unwrap_or(false)
                || bool_field(value, &["official_rows", "official_complete_rows"]).unwrap_or(false);
            snapshot.provider_pipeline_ready |=
                bool_field(value, &["provider_pipeline_ready", "provider_ready"]).unwrap_or(false);
            snapshot.kis_market_data_ready |= bool_field(
                value,
                &[
                    "kis_market_data_ready",
                    "market_data_ready",
                    "auth_ready",
                    "base_url_ready",
                ],
            )
            .unwrap_or(false)
                || string_field(value, &["candle_sufficiency_status"]).is_some_and(|status| {
                    status.contains("Sufficient") || status.contains("Ready")
                });
            snapshot.committee_trinity_ready |=
                bool_field(value, &["committee_trinity_ready", "committee_ready"]).unwrap_or(false);
            snapshot.chair_ready |=
                bool_field(value, &["chair_ready", "chair_v0_ready"]).unwrap_or(false);
            snapshot.risk_governor_ready |=
                bool_field(value, &["risk_governor_ready", "risk_governor_stable"])
                    .unwrap_or(false);
            snapshot.owner_input_ready |=
                bool_field(value, &["owner_input_ready", "owner_layer_ready"]).unwrap_or(false);
            snapshot.control_tower_ready |=
                bool_field(value, &["control_tower_ready", "control_tower_v1_ready"])
                    .unwrap_or(false)
                    || string_field(value, &["system_mode", "health_status"])
                        .is_some_and(|text| !text.trim().is_empty());
            snapshot.no_live_paths &= !bool_field(
                value,
                &["live_trading_path_present", "live_trading_enabled"],
            )
            .unwrap_or(false);
            snapshot.no_broker_paths &= !bool_field(
                value,
                &[
                    "broker_path_present",
                    "broker_surface_present",
                    "account_surface_present",
                ],
            )
            .unwrap_or(false);
            snapshot.mamba_runtime_present |=
                bool_field(value, &["mamba_runtime_present", "mamba3_runtime_present"])
                    .unwrap_or(false);
            snapshot.official_row_count = snapshot
                .official_row_count
                .max(usize_field(value, &["official_row_count", "official_rows"]));
            snapshot.complete_row_count = snapshot.complete_row_count.max(usize_field(
                value,
                &[
                    "complete_row_count",
                    "complete_rows",
                    "official_complete_rows",
                ],
            ));
            snapshot.outcome_links = snapshot.outcome_links.max(usize_field(
                value,
                &["outcome_links", "generated_outcome_links"],
            ));
            snapshot.counterfactuals = snapshot.counterfactuals.max(usize_field(
                value,
                &[
                    "counterfactuals",
                    "no_trade_counterfactuals",
                    "risk_denied_counterfactuals",
                ],
            ));
            snapshot.kis_evidence_depth_weak |=
                bool_field(value, &["kis_evidence_depth_weak", "evidence_depth_weak"])
                    .unwrap_or(false)
                    || string_field(value, &["sufficiency_status", "evidence_status"])
                        .is_some_and(|status| status.contains("NeedMoreKIS"));
            snapshot.outcome_link_depth_weak |=
                bool_field(value, &["outcome_link_depth_weak", "outcome_depth_weak"])
                    .unwrap_or(false)
                    || string_field(value, &["current_bottleneck", "primary_bottleneck"])
                        .is_some_and(|status| {
                            status.contains("OutcomeLinkDepth") || status.contains("OutcomeLinks")
                        });
            snapshot.counterfactual_depth_weak |= bool_field(
                value,
                &["counterfactual_depth_weak", "counterfactual_depth_needed"],
            )
            .unwrap_or(false);
            snapshot.risk_review_needed |=
                bool_field(value, &["risk_review_needed"]).unwrap_or(false);
            snapshot.control_tower_work_needed |=
                bool_field(value, &["control_tower_work_needed"]).unwrap_or(false);
        }
        snapshot
    }
}

fn bool_field(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|key| {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        matches.into_iter().find_map(|item| match item {
            Value::Bool(flag) => Some(*flag),
            Value::Number(number) => number.as_u64().map(|value| value > 0),
            Value::String(text) => match text.to_ascii_lowercase().as_str() {
                "true" | "ready" | "passed" | "sufficientforpaperonly" => Some(true),
                "false" | "missing" | "blocked" => Some(false),
                _ => None,
            },
            _ => None,
        })
    })
}

fn usize_field(value: &Value, keys: &[&str]) -> usize {
    keys.iter()
        .flat_map(|key| {
            let mut matches = Vec::new();
            collect_matches(value, key, &mut matches);
            matches
        })
        .filter_map(|item| match item {
            Value::Number(number) => number.as_u64().map(|value| value as usize),
            Value::String(text) => text.parse::<usize>().ok(),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        matches
            .into_iter()
            .find_map(|item| item.as_str().map(|text| text.to_string()))
    })
}

fn collect_matches<'a>(value: &'a Value, key: &str, output: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if let Some(item) = map.get(key) {
                output.push(item);
            }
            for child in map.values() {
                collect_matches(child, key, output);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_matches(child, key, output);
            }
        }
        _ => {}
    }
}
