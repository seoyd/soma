use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::experiment::{AiSignalStatus, OfficialAiBenchmarkReport, OfficialConsistencyStatus};

use super::sequence_spec::SequenceDatasetSpec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3RequirementStatus {
    Satisfied,
    PartiallySatisfied,
    Missing,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3FinRequirement {
    pub requirement_id: String,
    pub description: String,
    pub status: Mamba3RequirementStatus,
    pub evidence: Vec<String>,
    pub blockers: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3FinGapOverallStatus {
    NotReady,
    ExternalPredictionReady,
    SequenceDatasetReady,
    PrototypeSpecReady,
    ReadyForMamba3FinPrototype,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3FinGapAnalysisReport {
    pub requirements: Vec<Mamba3FinRequirement>,
    pub overall_status: Mamba3FinGapOverallStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingBackend {
    PythonExternal,
    RustDeferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceMode {
    ExternalPredictionFile,
    RustNativeDeferred,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3FinCandidateSpec {
    pub candidate_id: String,
    pub input_window_size: usize,
    pub d_model: usize,
    pub n_layers: usize,
    pub state_dim: usize,
    pub use_complex_state: bool,
    pub use_mimo: bool,
    pub use_micro_attention: bool,
    #[serde(default)]
    pub attention_interval_layers: Option<usize>,
    pub output_heads: Vec<String>,
    pub training_backend: TrainingBackend,
    pub inference_mode: InferenceMode,
    #[serde(default)]
    pub max_params_estimate: Option<usize>,
    #[serde(default)]
    pub max_latency_target_ms: Option<u64>,
    #[serde(default)]
    pub max_memory_mb: Option<usize>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Mamba3FinCandidateSpec {
    pub fn default_external() -> Self {
        Self {
            candidate_id: "Mamba3FinLiteExternal".to_string(),
            input_window_size: 64,
            d_model: 64,
            n_layers: 4,
            state_dim: 16,
            use_complex_state: true,
            use_mimo: true,
            use_micro_attention: false,
            attention_interval_layers: None,
            output_heads: vec![
                "p_win".to_string(),
                "p_stop".to_string(),
                "expected_return".to_string(),
                "expected_drawdown".to_string(),
                "confidence".to_string(),
                "no_trade_probability".to_string(),
                "horizon_bars".to_string(),
            ],
            training_backend: TrainingBackend::PythonExternal,
            inference_mode: InferenceMode::ExternalPredictionFile,
            max_params_estimate: Some(1_000_000),
            max_latency_target_ms: Some(100),
            max_memory_mb: Some(256),
            reason_codes: vec![ReasonCode::MambaCandidateSpecBuilt],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3FinCandidateReadiness {
    DoNotBuildYet,
    BuildExternalPrototype,
    BuildSequenceDatasetFirst,
    ImproveOfficialEvidenceFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3FinCandidateReport {
    pub candidate_spec: Mamba3FinCandidateSpec,
    pub readiness: Mamba3FinCandidateReadiness,
    pub blockers: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_mamba3fin_gap_analysis(
    benchmark: Option<&OfficialAiBenchmarkReport>,
    consistency_status: Option<OfficialConsistencyStatus>,
    sequence_spec: Option<&SequenceDatasetSpec>,
) -> Mamba3FinGapAnalysisReport {
    let official_evidence_strong = benchmark.is_some_and(|report| {
        matches!(
            report.usefulness_report.status,
            AiSignalStatus::BaselineEvaluated
                | AiSignalStatus::ExternalModelEvaluated
                | AiSignalStatus::UsefulCandidate
        )
    }) && matches!(
        consistency_status.unwrap_or(OfficialConsistencyStatus::NeedMoreExperiments),
        OfficialConsistencyStatus::ConsistentEnough
    );
    let sequence_ready = sequence_spec
        .as_ref()
        .is_some_and(|spec| spec.no_lookahead_safe && spec.storage_budget_ok);

    let requirements = vec![
        requirement(
            "SequenceFeatureFrame",
            "Current feature frame can be turned into fixed-length windows without lookahead.",
            if sequence_ready {
                Mamba3RequirementStatus::Satisfied
            } else if sequence_spec.is_some() {
                Mamba3RequirementStatus::PartiallySatisfied
            } else {
                Mamba3RequirementStatus::Missing
            },
            vec!["SequenceDatasetSpec available".to_string()],
            if sequence_spec.is_some() {
                Vec::new()
            } else {
                vec!["sequence dataset spec not built".to_string()]
            },
        ),
        requirement(
            "TradingHeads",
            "Prediction bridge can express trading heads needed by a future Mamba3Fin-lite model.",
            Mamba3RequirementStatus::Satisfied,
            vec!["PredictionRow already supports p_win/p_stop/return/drawdown/confidence/no_trade_probability/horizon_bars".to_string()],
            Vec::new(),
        ),
        requirement(
            "TripleBarrierLabels",
            "Current labels remain compatible with sequence windows and cost-aware evaluation.",
            if sequence_spec.is_some() {
                Mamba3RequirementStatus::Satisfied
            } else {
                Mamba3RequirementStatus::PartiallySatisfied
            },
            vec!["Triple barrier labels with fees/slippage already exist".to_string()],
            Vec::new(),
        ),
        requirement(
            "ExternalPredictionBridge",
            "External-first prediction import path is ready before any Rust-native inference.",
            Mamba3RequirementStatus::Satisfied,
            vec!["ExternalPredictionSignalModel and prediction CSV import already exist".to_string()],
            Vec::new(),
        ),
        requirement(
            "Calibration",
            "Current system reports Brier and ECE for p_win calibration.",
            Mamba3RequirementStatus::Satisfied,
            vec!["CalibrationReport already available".to_string()],
            Vec::new(),
        ),
        requirement(
            "RiskGovernorIntegration",
            "Any future model candidate must still route through Chair and Risk Governor.",
            Mamba3RequirementStatus::Satisfied,
            vec!["Risk Governor remains absolute veto".to_string()],
            Vec::new(),
        ),
        requirement(
            "StorageBudget",
            "Sequence export must stay bounded before any future prototype work.",
            if sequence_spec
                .as_ref()
                .is_some_and(|spec| spec.storage_budget_ok)
            {
                Mamba3RequirementStatus::Satisfied
            } else {
                Mamba3RequirementStatus::PartiallySatisfied
            },
            vec!["Storage budget reporting already exists".to_string()],
            if sequence_spec
                .as_ref()
                .is_some_and(|spec| spec.storage_budget_ok)
            {
                Vec::new()
            } else {
                vec!["sequence storage estimate not validated or exceeds budget".to_string()]
            },
        ),
        requirement(
            "TrainingRuntimeSeparation",
            "Training stays outside Rust runtime.",
            Mamba3RequirementStatus::Satisfied,
            vec!["Python research bridge remains optional and external".to_string()],
            Vec::new(),
        ),
        requirement(
            "InferenceRuntimePlan",
            "Inference can stay external-first instead of Rust-native now.",
            Mamba3RequirementStatus::Satisfied,
            vec!["ExternalPredictionFile is the intended prototype path".to_string()],
            Vec::new(),
        ),
        requirement(
            "Mamba3SpecificMath",
            "Current runtime implements Mamba3-style recurrence, complex state, MIMO, and streaming state.",
            Mamba3RequirementStatus::Missing,
            Vec::new(),
            vec![
                "expressive recurrence missing".to_string(),
                "complex-valued state update missing".to_string(),
                "MIMO SSM missing".to_string(),
                "hardware-aware scan missing".to_string(),
                "streaming state API missing".to_string(),
            ],
        ),
    ];
    let blockers = requirements
        .iter()
        .filter(|item| matches!(item.status, Mamba3RequirementStatus::Missing))
        .flat_map(|item| item.blockers.clone())
        .collect::<Vec<_>>();
    let mut warnings = Vec::new();
    if !official_evidence_strong {
        warnings
            .push("official evidence is not yet strong enough for model escalation".to_string());
    }
    if !sequence_ready {
        warnings.push("sequence dataset export/spec is not ready enough yet".to_string());
    }
    let overall_status = if !official_evidence_strong {
        Mamba3FinGapOverallStatus::ExternalPredictionReady
    } else if !sequence_ready {
        Mamba3FinGapOverallStatus::ExternalPredictionReady
    } else if matches!(
        consistency_status.unwrap_or(OfficialConsistencyStatus::NeedMoreExperiments),
        OfficialConsistencyStatus::ConsistentEnough
    ) {
        Mamba3FinGapOverallStatus::PrototypeSpecReady
    } else {
        Mamba3FinGapOverallStatus::SequenceDatasetReady
    };

    Mamba3FinGapAnalysisReport {
        requirements,
        overall_status,
        blockers,
        warnings,
        reason_codes: vec![ReasonCode::MambaGapAnalysisBuilt],
    }
}

pub fn build_mamba3fin_candidate_report(
    gap_analysis: &Mamba3FinGapAnalysisReport,
    consistency_status: OfficialConsistencyStatus,
    usefulness_status: Option<AiSignalStatus>,
    sequence_spec: Option<&SequenceDatasetSpec>,
) -> Mamba3FinCandidateReport {
    let readiness = if matches!(
        consistency_status,
        OfficialConsistencyStatus::MissingAuth
            | OfficialConsistencyStatus::MissingEquityData
            | OfficialConsistencyStatus::InsufficientOutcomes
            | OfficialConsistencyStatus::NeedMoreExperiments
    ) {
        Mamba3FinCandidateReadiness::ImproveOfficialEvidenceFirst
    } else if matches!(
        usefulness_status.unwrap_or(AiSignalStatus::PipelineOnly),
        AiSignalStatus::PoorCalibration
            | AiSignalStatus::WorseThanBaseline
            | AiSignalStatus::InsufficientOutcomes
            | AiSignalStatus::Blocked
    ) {
        Mamba3FinCandidateReadiness::ImproveSignalModelFirst
    } else if matches!(
        usefulness_status.unwrap_or(AiSignalStatus::PipelineOnly),
        AiSignalStatus::PoorRiskBehavior | AiSignalStatus::RejectedByRisk
    ) {
        Mamba3FinCandidateReadiness::ImproveRiskGovernorFirst
    } else if sequence_spec.is_none()
        || !sequence_spec
            .as_ref()
            .is_some_and(|spec| spec.storage_budget_ok && spec.no_lookahead_safe)
    {
        Mamba3FinCandidateReadiness::BuildSequenceDatasetFirst
    } else if matches!(
        gap_analysis.overall_status,
        Mamba3FinGapOverallStatus::PrototypeSpecReady
            | Mamba3FinGapOverallStatus::ReadyForMamba3FinPrototype
    ) || (matches!(consistency_status, OfficialConsistencyStatus::CryptoOnly)
        && matches!(
            usefulness_status.unwrap_or(AiSignalStatus::PipelineOnly),
            AiSignalStatus::ExternalModelEvaluated | AiSignalStatus::UsefulCandidate
        ))
    {
        Mamba3FinCandidateReadiness::BuildExternalPrototype
    } else {
        Mamba3FinCandidateReadiness::DoNotBuildYet
    };
    Mamba3FinCandidateReport {
        candidate_spec: Mamba3FinCandidateSpec::default_external(),
        readiness,
        blockers: gap_analysis.blockers.clone(),
        reason_codes: vec![ReasonCode::MambaCandidateSpecBuilt],
    }
}

impl Mamba3FinGapAnalysisReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![format!("overall_status={:?}", self.overall_status)];
        for requirement in &self.requirements {
            lines.push(format!(
                "requirement={};status={:?};blockers={};evidence={}",
                requirement.requirement_id,
                requirement.status,
                requirement.blockers.join("|"),
                requirement.evidence.join("|")
            ));
        }
        for blocker in &self.blockers {
            lines.push(format!("blocker={blocker}"));
        }
        for warning in &self.warnings {
            lines.push(format!("warning={warning}"));
        }
        lines.join("\n")
    }
}

impl Mamba3FinCandidateReport {
    pub fn to_text(&self) -> String {
        [
            format!("candidate_id={}", self.candidate_spec.candidate_id),
            format!("readiness={:?}", self.readiness),
            format!(
                "training_backend={:?}",
                self.candidate_spec.training_backend
            ),
            format!("inference_mode={:?}", self.candidate_spec.inference_mode),
            format!("blockers={}", self.blockers.join(" | ")),
        ]
        .join("\n")
    }
}

fn requirement(
    requirement_id: &str,
    description: &str,
    status: Mamba3RequirementStatus,
    evidence: Vec<String>,
    blockers: Vec<String>,
) -> Mamba3FinRequirement {
    Mamba3FinRequirement {
        requirement_id: requirement_id.to_string(),
        description: description.to_string(),
        status,
        evidence,
        blockers,
        reason_codes: vec![match status {
            Mamba3RequirementStatus::Satisfied => ReasonCode::MambaRequirementSatisfied,
            Mamba3RequirementStatus::PartiallySatisfied => {
                ReasonCode::MambaRequirementPartiallySatisfied
            }
            Mamba3RequirementStatus::Missing => ReasonCode::MambaRequirementMissing,
            Mamba3RequirementStatus::Deferred => ReasonCode::MambaRequirementDeferred,
        }],
    }
}
