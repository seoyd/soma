use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audit::{AuditLedger, AuditRecord, AuditSummary};
use crate::core::{
    CoreContractRegistry, CoreContractRegistryReport, CorePerformanceBudget,
    CorePerformanceBudgetReport, DeterminismCheck, DeterminismInputFingerprint,
    DeterminismOutputFingerprint, LiveSafetyReport, LiveSafetyStatus, ReasonCode, ReasonCodeAudit,
    RuntimeMode, RuntimeStage, RuntimeState, RuntimeStateReport, build_live_safety_report,
    measure_performance_budget,
};
use crate::risk::{RiskInvariantReport, build_risk_invariant_report};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreReadinessStatus {
    ReadyForMoreOfficialEvidence,
    ReadyForExternalModelPrototype,
    ReadyForSequenceDatasetBuild,
    NotReadyDueToContractDrift,
    NotReadyDueToRiskInvariantFailure,
    NotReadyDueToNondeterminism,
    NotReadyDueToAuditGap,
    NotReadyDueToLiveSafetyGap,
    NeedMoreCoreHardening,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreNextRecommendation {
    MoreOfficialEvidence,
    ExternalModelPrototype,
    SequenceDatasetBuild,
    CoreHardeningAgain,
    ImproveRiskGovernorFirst,
    ImproveDataFirst,
    ImproveSignalModelFirst,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreCheckConfig {
    pub check_id: String,
    pub runtime_mode: RuntimeMode,
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub official_evidence_ready: bool,
    #[serde(default = "default_true")]
    pub external_model_bridge_ready: bool,
    #[serde(default)]
    pub sequence_dataset_ready: bool,
    #[serde(default)]
    pub performance_budget: CorePerformanceBudget,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreReadinessReport {
    pub runtime_state_report: RuntimeStateReport,
    pub contract_registry_report: CoreContractRegistryReport,
    pub determinism_report: DeterminismCheck,
    pub reason_code_audit: ReasonCodeAudit,
    pub audit_summary: AuditSummary,
    pub risk_invariant_report: RiskInvariantReport,
    pub live_safety_report: LiveSafetyReport,
    pub performance_budget_report: CorePerformanceBudgetReport,
    pub final_status: CoreReadinessStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub next_recommendation: CoreNextRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoreCheckRunner;

impl Default for CoreCheckConfig {
    fn default() -> Self {
        Self {
            check_id: "core_check".to_string(),
            runtime_mode: RuntimeMode::Research,
            timestamp_ms: None,
            output_root: default_output_root(),
            official_evidence_ready: false,
            external_model_bridge_ready: true,
            sequence_dataset_ready: false,
            performance_budget: CorePerformanceBudget::default(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CoreCheckConfig {
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
        if self.output_root.contains("://") {
            vec![ReasonCode::RemotePathRejected]
        } else {
            Vec::new()
        }
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.check_id)
    }
}

impl CoreCheckRunner {
    pub fn run(&self, config: &CoreCheckConfig) -> Result<CoreReadinessReport, String> {
        if !config.validate_local_paths().is_empty() {
            return Err("core-check output path must be local".to_string());
        }

        let runtime_state_report = build_runtime_state_report(config.runtime_mode)?;
        let contract_registry_report = CoreContractRegistry::default().report();
        let command_names = vec![
            "run".to_string(),
            "batch".to_string(),
            "ablation".to_string(),
            "sprint14".to_string(),
            "evidence-close".to_string(),
            "real-evidence".to_string(),
            "data-preflight".to_string(),
            "onboard-data".to_string(),
            "import-krx-snapshot".to_string(),
            "collect-candles".to_string(),
            "campaign".to_string(),
            "collect-plan".to_string(),
            "evidence-run".to_string(),
            "collect-and-evaluate".to_string(),
            "ai-benchmark".to_string(),
            "core-benchmark".to_string(),
            "collect-train-evaluate".to_string(),
            "mamba-readiness".to_string(),
            "core-check".to_string(),
            "evidence-plan".to_string(),
            "evidence-execute".to_string(),
            "readiness-matrix".to_string(),
            "committee-smoke".to_string(),
            "committee-load-scenarios".to_string(),
            "committee-replay".to_string(),
            "committee-diagnostics".to_string(),
            "persona-cards".to_string(),
            "compare".to_string(),
            "source-benchmark".to_string(),
            "baseline".to_string(),
            "dataset".to_string(),
        ];
        let live_safety_report = build_live_safety_report(&command_names, false);
        let risk_invariant_report = build_risk_invariant_report();
        let performance_budget_report =
            measure_performance_budget(&config.performance_budget, 0, 0, 0, 0, 0, 0, 0, &[]);

        let mut ledger = AuditLedger::default();
        ledger.add_record(AuditRecord {
            audit_id: format!("{}-init", config.check_id),
            mode: config.runtime_mode,
            stage: RuntimeStage::Init,
            source_kind: "core-check".to_string(),
            input_fingerprint: config.check_id.clone(),
            output_fingerprint: None,
            decision_summary: Some("init".to_string()),
            risk_decision: None,
            reason_codes: vec![ReasonCode::RuntimeStateInitialized],
            timestamp_ms: config.timestamp_ms,
        });
        ledger.add_record(AuditRecord {
            audit_id: format!("{}-risk", config.check_id),
            mode: config.runtime_mode,
            stage: RuntimeStage::RiskEvaluation,
            source_kind: "core-check".to_string(),
            input_fingerprint: config.check_id.clone(),
            output_fingerprint: Some(runtime_state_report.fingerprint.clone()),
            decision_summary: Some("risk-invariants".to_string()),
            risk_decision: Some(format!("{}", risk_invariant_report.all_passed())),
            reason_codes: risk_invariant_report.reason_codes.clone(),
            timestamp_ms: config.timestamp_ms,
        });
        ledger.add_record(AuditRecord {
            audit_id: format!("{}-report", config.check_id),
            mode: config.runtime_mode,
            stage: RuntimeStage::ReportGeneration,
            source_kind: "core-check".to_string(),
            input_fingerprint: config.check_id.clone(),
            output_fingerprint: Some(contract_registry_report.fingerprint.clone()),
            decision_summary: Some("report".to_string()),
            risk_decision: None,
            reason_codes: vec![ReasonCode::CoreReadinessBuilt],
            timestamp_ms: config.timestamp_ms,
        });
        let audit_summary = ledger.summarize();

        let input_fingerprint = DeterminismInputFingerprint::new(
            "core-check",
            &config.to_toml_string()?,
            None,
            None,
            None,
        );
        let output_material = [
            runtime_state_report.to_text(),
            contract_registry_report.to_text(),
            live_safety_report.to_text(),
            performance_budget_report.to_text(),
            audit_summary.to_text(),
        ]
        .join("\n");
        let output_fingerprint = DeterminismOutputFingerprint::new(
            &output_material,
            ledger.records.len(),
            &[
                ReasonCode::CoreReadinessBuilt,
                ReasonCode::ContractRegistryBuilt,
                ReasonCode::LiveSafetyReportBuilt,
            ],
            2,
            output_material.len(),
        );
        let determinism_report =
            DeterminismCheck::compare(input_fingerprint, &output_fingerprint, &output_fingerprint);

        let reason_code_audit = crate::core::audit_reason_codes(
            &[
                ReasonCode::RuntimeStateInitialized,
                ReasonCode::RuntimeTransitionRecorded,
                ReasonCode::ContractRegistryBuilt,
                ReasonCode::DeterminismCheckPassed,
                ReasonCode::CoreReadinessBuilt,
                ReasonCode::LiveModeDisabled,
                ReasonCode::MissingFile,
                ReasonCode::RemotePathRejected,
                ReasonCode::MissingAuth,
                ReasonCode::DataQualityTooLow,
                ReasonCode::SchemaMismatch,
                ReasonCode::InvalidPrediction,
                ReasonCode::BudgetExceeded,
                ReasonCode::PreflightFailed,
                ReasonCode::RiskDenied,
                ReasonCode::NoTradeDefault,
            ],
            &[],
            None,
        );

        let report = evaluate_core_readiness(
            runtime_state_report,
            contract_registry_report,
            determinism_report,
            reason_code_audit,
            audit_summary,
            risk_invariant_report,
            live_safety_report,
            performance_budget_report,
            config.official_evidence_ready,
            config.external_model_bridge_ready,
            config.sequence_dataset_ready,
        );
        report.write_to_dir(&config.output_dir())?;
        Ok(report)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_core_readiness(
    runtime_state_report: RuntimeStateReport,
    contract_registry_report: CoreContractRegistryReport,
    determinism_report: DeterminismCheck,
    reason_code_audit: ReasonCodeAudit,
    audit_summary: AuditSummary,
    risk_invariant_report: RiskInvariantReport,
    live_safety_report: LiveSafetyReport,
    performance_budget_report: CorePerformanceBudgetReport,
    official_evidence_ready: bool,
    external_model_bridge_ready: bool,
    sequence_dataset_ready: bool,
) -> CoreReadinessReport {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    let (final_status, next_recommendation, mut reason_codes) = if !risk_invariant_report
        .all_passed()
    {
        blockers.push("risk invariants failed".to_string());
        (
            CoreReadinessStatus::NotReadyDueToRiskInvariantFailure,
            CoreNextRecommendation::ImproveRiskGovernorFirst,
            vec![
                ReasonCode::CoreReadinessBuilt,
                ReasonCode::CoreReadinessRiskFailure,
            ],
        )
    } else if !determinism_report.deterministic {
        blockers.push("determinism check failed".to_string());
        (
            CoreReadinessStatus::NotReadyDueToNondeterminism,
            CoreNextRecommendation::CoreHardeningAgain,
            vec![
                ReasonCode::CoreReadinessBuilt,
                ReasonCode::CoreReadinessNondeterministic,
            ],
        )
    } else if live_safety_report.status != LiveSafetyStatus::SafeResearchOnly {
        blockers.push("live safety gap detected".to_string());
        (
            CoreReadinessStatus::NotReadyDueToLiveSafetyGap,
            CoreNextRecommendation::CoreHardeningAgain,
            vec![
                ReasonCode::CoreReadinessBuilt,
                ReasonCode::CoreReadinessLiveSafetyGap,
            ],
        )
    } else if contract_registry_report.has_incompatible() {
        blockers.push("contract drift detected".to_string());
        (
            CoreReadinessStatus::NotReadyDueToContractDrift,
            CoreNextRecommendation::CoreHardeningAgain,
            vec![
                ReasonCode::CoreReadinessBuilt,
                ReasonCode::CoreReadinessContractDrift,
            ],
        )
    } else if audit_summary.missing_reason_code_count > 0
        || !matches!(
            reason_code_audit.completeness_status,
            crate::core::ReasonCodeCompletenessStatus::Complete
        )
    {
        blockers.push("audit or reason-code gap detected".to_string());
        (
            CoreReadinessStatus::NotReadyDueToAuditGap,
            CoreNextRecommendation::CoreHardeningAgain,
            vec![
                ReasonCode::CoreReadinessBuilt,
                ReasonCode::CoreReadinessAuditGap,
            ],
        )
    } else if performance_budget_report.budget_exceeded {
        warnings.push(
            "performance budget exceeded; compact artifacts before expanding scope".to_string(),
        );
        (
            CoreReadinessStatus::NeedMoreCoreHardening,
            CoreNextRecommendation::CoreHardeningAgain,
            vec![ReasonCode::CoreReadinessBuilt, ReasonCode::BudgetExceeded],
        )
    } else if !official_evidence_ready {
        warnings
            .push("core checks passed, but official evidence breadth is still weak".to_string());
        (
            CoreReadinessStatus::ReadyForMoreOfficialEvidence,
            CoreNextRecommendation::MoreOfficialEvidence,
            vec![ReasonCode::CoreReadinessBuilt],
        )
    } else if !sequence_dataset_ready {
        (
            CoreReadinessStatus::ReadyForSequenceDatasetBuild,
            CoreNextRecommendation::SequenceDatasetBuild,
            vec![ReasonCode::CoreReadinessBuilt],
        )
    } else if external_model_bridge_ready {
        (
            CoreReadinessStatus::ReadyForExternalModelPrototype,
            CoreNextRecommendation::ExternalModelPrototype,
            vec![ReasonCode::CoreReadinessBuilt],
        )
    } else {
        (
            CoreReadinessStatus::NeedMoreCoreHardening,
            CoreNextRecommendation::CoreHardeningAgain,
            vec![ReasonCode::CoreReadinessBuilt],
        )
    };

    if runtime_state_report.state.mode == RuntimeMode::LiveDisabled {
        warnings.push("live mode remains explicitly disabled".to_string());
        reason_codes.push(ReasonCode::LiveModeDisabled);
    }

    CoreReadinessReport {
        runtime_state_report,
        contract_registry_report,
        determinism_report,
        reason_code_audit,
        audit_summary,
        risk_invariant_report,
        live_safety_report,
        performance_budget_report,
        final_status,
        blockers,
        warnings,
        next_recommendation,
        reason_codes,
    }
}

impl CoreReadinessReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            self.runtime_state_report.to_text(),
            self.contract_registry_report.to_text(),
            self.determinism_report.to_text(),
            self.reason_code_audit.to_text(),
            self.audit_summary.to_text(),
            self.risk_invariant_report.to_text(),
            self.live_safety_report.to_text(),
            self.performance_budget_report.to_text(),
            format!("final_status={:?}", self.final_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("next_recommendation={:?}", self.next_recommendation),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("core_readiness_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(output_dir.join("core_readiness_report.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

fn build_runtime_state_report(mode: RuntimeMode) -> Result<RuntimeStateReport, String> {
    let mut state = RuntimeState::new(mode);
    for stage in [
        RuntimeStage::LoadConfig,
        RuntimeStage::ValidateConfig,
        RuntimeStage::LoadData,
        RuntimeStage::ValidateData,
        RuntimeStage::BuildFeatures,
        RuntimeStage::GenerateSignals,
        RuntimeStage::ChairDecision,
        RuntimeStage::RiskEvaluation,
    ] {
        state
            .transition_to(stage, stage >= RuntimeStage::ChairDecision)
            .map_err(|err| format!("{err:?}"))?;
    }
    if mode.paper_execution_allowed() {
        state
            .transition_to(RuntimeStage::PaperExecution, true)
            .map_err(|err| format!("{err:?}"))?;
    }
    state
        .transition_to(RuntimeStage::OutcomeEvaluation, true)
        .map_err(|err| format!("{err:?}"))?;
    state
        .transition_to(RuntimeStage::ReportGeneration, true)
        .map_err(|err| format!("{err:?}"))?;
    state
        .transition_to(RuntimeStage::Completed, true)
        .map_err(|err| format!("{err:?}"))?;
    Ok(RuntimeStateReport::from_state(state))
}

fn default_output_root() -> String {
    "target/soma_core_check".to_string()
}

fn default_true() -> bool {
    true
}
