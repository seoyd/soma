use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, Regime, stable_hash_string, stable_reason_codes};
use crate::data::ProviderMarket;

use super::candidate_lifecycle::CandidateLifecycleStatus;

fn default_timestamp_ms() -> u64 {
    1_715_000_000_000
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateSourceKind {
    #[default]
    KISOfficialEvidence,
    OfficialEvidenceScaleOut,
    OfficialEvidenceDiversitySweep,
    CorePerformanceScorecard,
    CommitteeBenchmark,
    OwnerWatchlist,
    ResearchOnly,
    DiagnosticOnly,
    CryptoOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateEvidenceClass {
    #[default]
    Official,
    ResearchOnly,
    DiagnosticOnly,
    CryptoOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateGenerationStatus {
    CandidatesGenerated,
    #[default]
    NoCandidates,
    NeedMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidateGenerationInput {
    pub source_kind: CandidateSourceKind,
    pub source_report_path: String,
    pub symbol: String,
    pub market: ProviderMarket,
    pub timeframe: String,
    pub horizon_bars: u32,
    pub evidence_status: String,
    #[serde(default)]
    pub data_quality_score: Option<f64>,
    #[serde(default)]
    pub outcome_link_count: Option<usize>,
    #[serde(default)]
    pub counterfactual_count: Option<usize>,
    #[serde(default)]
    pub signal_summary: Option<String>,
    #[serde(default)]
    pub expected_edge: Option<f64>,
    #[serde(default)]
    pub expected_drawdown: Option<f64>,
    #[serde(default = "default_timestamp_ms")]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub spread_bps: Option<f64>,
    #[serde(default)]
    pub trade_value: Option<f64>,
    #[serde(default)]
    pub regime: Option<Regime>,
    #[serde(default)]
    pub paper_outcome_hint: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratedCandidate {
    pub candidate_id: String,
    pub symbol: String,
    pub market: ProviderMarket,
    pub timeframe: String,
    pub horizon_bars: u32,
    pub source_kind: CandidateSourceKind,
    pub evidence_class: CandidateEvidenceClass,
    pub initial_status: CandidateLifecycleStatus,
    #[serde(default)]
    pub expected_edge: Option<f64>,
    #[serde(default)]
    pub expected_drawdown: Option<f64>,
    #[serde(default)]
    pub data_quality_score: Option<f64>,
    #[serde(default)]
    pub signal_summary: Option<String>,
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub spread_bps: Option<f64>,
    #[serde(default)]
    pub trade_value: Option<f64>,
    #[serde(default)]
    pub regime: Option<Regime>,
    #[serde(default)]
    pub paper_outcome_hint: Option<String>,
    #[serde(default)]
    pub source_report_path: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedCandidate {
    pub source_report_path: String,
    pub symbol: String,
    pub summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CandidateGenerationReport {
    #[serde(default)]
    pub generated_candidates: Vec<GeneratedCandidate>,
    #[serde(default)]
    pub skipped_candidates: Vec<SkippedCandidate>,
    pub official_candidates: usize,
    pub research_only_candidates: usize,
    pub diagnostic_candidates: usize,
    pub crypto_candidates: usize,
    pub generation_status: CandidateGenerationStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateGenerationSettings {
    pub require_official_evidence_for_official_candidates: bool,
    pub allow_research_only_candidates: bool,
    pub allow_diagnostic_candidates: bool,
    pub allow_crypto_only_candidates: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandidateGenerationFromEvidence;

impl CandidateGenerationReport {
    pub fn stabilize(&mut self) {
        self.generated_candidates
            .sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        self.skipped_candidates.sort_by(|left, right| {
            left.symbol
                .cmp(&right.symbol)
                .then(left.source_report_path.cmp(&right.source_report_path))
        });
        for candidate in &mut self.generated_candidates {
            candidate.reason_codes = stable_reason_codes(&candidate.reason_codes);
        }
        for skipped in &mut self.skipped_candidates {
            skipped.reason_codes = stable_reason_codes(&skipped.reason_codes);
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=candidate generation is local deterministic evidence triage only"
                .to_string(),
            "paper_only_warning=generated candidates are paper-only research artifacts and never imply real execution".to_string(),
            format!("generated_candidates={}", self.generated_candidates.len()),
            format!("skipped_candidates={}", self.skipped_candidates.len()),
            format!("official_candidates={}", self.official_candidates),
            format!("research_only_candidates={}", self.research_only_candidates),
            format!("diagnostic_candidates={}", self.diagnostic_candidates),
            format!("crypto_candidates={}", self.crypto_candidates),
            format!("generation_status={:?}", self.generation_status),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}

impl CandidateGenerationFromEvidence {
    pub fn load_inputs_from_paths(
        paths: &[String],
    ) -> Result<Vec<CandidateGenerationInput>, String> {
        let mut all_inputs = Vec::new();
        for path in paths {
            if path.contains("://") {
                return Err("candidate generation input paths must be local".to_string());
            }
            let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
            if contents.trim_start().starts_with('[') {
                let mut inputs = serde_json::from_str::<Vec<CandidateGenerationInput>>(&contents)
                    .map_err(|err| err.to_string())?;
                all_inputs.append(&mut inputs);
            } else {
                all_inputs.push(
                    serde_json::from_str::<CandidateGenerationInput>(&contents)
                        .map_err(|err| err.to_string())?,
                );
            }
        }
        all_inputs.sort_by(|left, right| {
            left.symbol
                .cmp(&right.symbol)
                .then(left.source_report_path.cmp(&right.source_report_path))
                .then(left.timeframe.cmp(&right.timeframe))
        });
        Ok(all_inputs)
    }

    pub fn generate(
        &self,
        inputs: &[CandidateGenerationInput],
        settings: &CandidateGenerationSettings,
    ) -> CandidateGenerationReport {
        let mut report = CandidateGenerationReport {
            generated_candidates: Vec::new(),
            skipped_candidates: Vec::new(),
            official_candidates: 0,
            research_only_candidates: 0,
            diagnostic_candidates: 0,
            crypto_candidates: 0,
            generation_status: CandidateGenerationStatus::NoCandidates,
            reason_codes: vec![ReasonCode::DeterministicPath],
            fingerprint: String::new(),
        };

        for input in inputs {
            let evidence_class = classify_evidence_class(input.source_kind, input.market);
            if is_weak_evidence(input, evidence_class, settings) {
                report.skipped_candidates.push(SkippedCandidate {
                    source_report_path: input.source_report_path.clone(),
                    symbol: input.symbol.clone(),
                    summary: input
                        .signal_summary
                        .clone()
                        .unwrap_or_else(|| "weak evidence".to_string()),
                    reason_codes: stable_reason_codes(&[
                        ReasonCode::EvidenceGapDetected,
                        ReasonCode::EvidenceStillInsufficient,
                    ]),
                });
                continue;
            }
            if matches!(evidence_class, CandidateEvidenceClass::ResearchOnly)
                && !settings.allow_research_only_candidates
            {
                report.skipped_candidates.push(SkippedCandidate {
                    source_report_path: input.source_report_path.clone(),
                    symbol: input.symbol.clone(),
                    summary: "research-only candidates disabled by config".to_string(),
                    reason_codes: vec![ReasonCode::ResearchOnlyOverride],
                });
                continue;
            }
            if matches!(evidence_class, CandidateEvidenceClass::DiagnosticOnly)
                && !settings.allow_diagnostic_candidates
            {
                report.skipped_candidates.push(SkippedCandidate {
                    source_report_path: input.source_report_path.clone(),
                    symbol: input.symbol.clone(),
                    summary: "diagnostic candidates disabled by config".to_string(),
                    reason_codes: vec![ReasonCode::ResearchOnlyOverride],
                });
                continue;
            }
            if matches!(evidence_class, CandidateEvidenceClass::CryptoOnly)
                && !settings.allow_crypto_only_candidates
            {
                report.skipped_candidates.push(SkippedCandidate {
                    source_report_path: input.source_report_path.clone(),
                    symbol: input.symbol.clone(),
                    summary: "crypto-only candidates disabled by config".to_string(),
                    reason_codes: vec![ReasonCode::ResearchOnlyOverride],
                });
                continue;
            }

            let initial_status = match evidence_class {
                CandidateEvidenceClass::Official => CandidateLifecycleStatus::EvidenceReady,
                CandidateEvidenceClass::ResearchOnly | CandidateEvidenceClass::CryptoOnly => {
                    CandidateLifecycleStatus::ResearchOnly
                }
                CandidateEvidenceClass::DiagnosticOnly => CandidateLifecycleStatus::DiagnosticOnly,
            };
            let candidate_id = stable_hash_string(&format!(
                "{:?}|{}|{:?}|{}|{}|{}",
                input.source_kind,
                input.symbol,
                input.market,
                input.timeframe,
                input.horizon_bars,
                input.source_report_path
            ));
            let mut reason_codes = input.reason_codes.clone();
            reason_codes.push(ReasonCode::DeterministicPath);
            if !matches!(evidence_class, CandidateEvidenceClass::Official) {
                reason_codes.push(ReasonCode::ResearchOnlyOverride);
            }
            report.generated_candidates.push(GeneratedCandidate {
                candidate_id,
                symbol: input.symbol.clone(),
                market: input.market,
                timeframe: input.timeframe.clone(),
                horizon_bars: input.horizon_bars,
                source_kind: input.source_kind,
                evidence_class,
                initial_status,
                expected_edge: input.expected_edge,
                expected_drawdown: input.expected_drawdown,
                data_quality_score: input.data_quality_score,
                signal_summary: input.signal_summary.clone(),
                timestamp_ms: input.timestamp_ms,
                confidence: input.confidence,
                spread_bps: input.spread_bps,
                trade_value: input.trade_value,
                regime: input.regime,
                paper_outcome_hint: input.paper_outcome_hint.clone(),
                source_report_path: Some(input.source_report_path.clone()),
                reason_codes: stable_reason_codes(&reason_codes),
            });
            match evidence_class {
                CandidateEvidenceClass::Official => report.official_candidates += 1,
                CandidateEvidenceClass::ResearchOnly => report.research_only_candidates += 1,
                CandidateEvidenceClass::DiagnosticOnly => report.diagnostic_candidates += 1,
                CandidateEvidenceClass::CryptoOnly => report.crypto_candidates += 1,
            }
        }

        report.generation_status = if report.generated_candidates.is_empty() {
            if report.skipped_candidates.is_empty() {
                CandidateGenerationStatus::NoCandidates
            } else {
                CandidateGenerationStatus::NeedMoreEvidence
            }
        } else if report.official_candidates == 0
            && report.diagnostic_candidates == report.generated_candidates.len()
        {
            CandidateGenerationStatus::DiagnosticOnly
        } else {
            CandidateGenerationStatus::CandidatesGenerated
        };
        report.stabilize();
        report
    }
}

fn classify_evidence_class(
    source_kind: CandidateSourceKind,
    market: ProviderMarket,
) -> CandidateEvidenceClass {
    match source_kind {
        CandidateSourceKind::KISOfficialEvidence
        | CandidateSourceKind::OfficialEvidenceScaleOut
        | CandidateSourceKind::OfficialEvidenceDiversitySweep
        | CandidateSourceKind::CorePerformanceScorecard
        | CandidateSourceKind::CommitteeBenchmark => CandidateEvidenceClass::Official,
        CandidateSourceKind::ResearchOnly | CandidateSourceKind::OwnerWatchlist => {
            CandidateEvidenceClass::ResearchOnly
        }
        CandidateSourceKind::DiagnosticOnly => CandidateEvidenceClass::DiagnosticOnly,
        CandidateSourceKind::CryptoOnly => {
            if matches!(market, ProviderMarket::Crypto) {
                CandidateEvidenceClass::CryptoOnly
            } else {
                CandidateEvidenceClass::ResearchOnly
            }
        }
    }
}

fn is_weak_evidence(
    input: &CandidateGenerationInput,
    evidence_class: CandidateEvidenceClass,
    settings: &CandidateGenerationSettings,
) -> bool {
    let status = input.evidence_status.to_ascii_lowercase();
    if status.contains("weak") || status.contains("need") || status.contains("insufficient") {
        return true;
    }
    if input.data_quality_score.unwrap_or(0.0) < 0.55 {
        return true;
    }
    if matches!(evidence_class, CandidateEvidenceClass::Official)
        && settings.require_official_evidence_for_official_candidates
        && input.outcome_link_count.unwrap_or(0) == 0
        && input.counterfactual_count.unwrap_or(0) == 0
    {
        return true;
    }
    false
}

pub fn write_candidate_generation_report(
    output_path: &Path,
    report: &CandidateGenerationReport,
) -> Result<(), String> {
    if output_path.to_string_lossy().contains("://") {
        return Err("candidate generation output path must be local".to_string());
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        output_path,
        serde_json::to_string_pretty(report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}
