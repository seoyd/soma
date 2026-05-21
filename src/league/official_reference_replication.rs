use std::fs;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_outcome_linker::{
    CommitteeOutcomeLinkSummary, OutcomeLinkedCommitteeScenarioPack,
    OutcomeLinkedCommitteeScenarioRow,
};
use super::committee_reference_pack::{
    CommitteeReferencePackConfig, GeneratedCommitteeReferencePack, GeneratedReferenceKind,
};
use super::committee_reference_pack_bundle::CommitteeReferencePackBundle;
use super::official_candle_coverage::{OfficialCandleCoverageReport, OfficialCandleCoverageStatus};
use super::official_evidence_replication::OfficialEvidenceReplicationConfig;
use super::official_row_injection::{
    OfficialEvidenceBoundary, OfficialRowInjectionResult, classify_row_boundary,
};
use super::reference_pack_quality::{ReferencePackQualityReport, ReferencePackQualityStatus};
use super::sufficiency_closure::SufficiencyClosureReport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialReferenceReplicationStatus {
    OfficialReferencesGenerated,
    ControlledReferencesOnly,
    CryptoOnlyReferences,
    MissingOfficialCandleData,
    MissingOutcomeWindows,
    MissingBaselineReferences,
    MissingCounterfactualDepth,
    ReferenceGenerationBlocked,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReferenceReplicationReport {
    pub replication_id: String,
    #[serde(default)]
    pub generated_reference_pack: Option<GeneratedCommitteeReferencePack>,
    #[serde(default)]
    pub reference_pack_quality: Option<ReferencePackQualityReport>,
    pub outcome_reference_count: usize,
    pub baseline_reference_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub official_ready_reference_count: usize,
    pub controlled_reference_count: usize,
    pub crypto_only_reference_count: usize,
    pub research_only_reference_count: usize,
    pub diagnostic_only_reference_count: usize,
    pub replication_status: OfficialReferenceReplicationStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReferenceReplicationArtifacts {
    pub report: OfficialReferenceReplicationReport,
    #[serde(default)]
    pub bundle: Option<CommitteeReferencePackBundle>,
    #[serde(default)]
    pub linked_pack: Option<OutcomeLinkedCommitteeScenarioPack>,
    #[serde(default)]
    pub closure_report: Option<SufficiencyClosureReport>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialReferenceReplicationRunner;

impl OfficialReferenceReplicationRunner {
    pub fn run(
        &self,
        config: &OfficialEvidenceReplicationConfig,
        row_injection: &OfficialRowInjectionResult,
        candle_series_paths: &[String],
        coverage_report: &OfficialCandleCoverageReport,
    ) -> Result<OfficialReferenceReplicationArtifacts, String> {
        if row_injection.injected_rows.is_empty() {
            return Ok(blocked_report(
                config,
                OfficialReferenceReplicationStatus::ReferenceGenerationBlocked,
            ));
        }
        if candle_series_paths.is_empty()
            || matches!(
                coverage_report.coverage_status,
                OfficialCandleCoverageStatus::MissingOfficialCandles
            )
        {
            return Ok(blocked_report(
                config,
                OfficialReferenceReplicationStatus::MissingOfficialCandleData,
            ));
        }
        let scenario_set =
            row_injection.to_scenario_set(&format!("{}-rows", config.replication_id));
        let scenario_dir = config.output_dir().join("reference_inputs");
        fs::create_dir_all(&scenario_dir).map_err(|err| err.to_string())?;
        let scenario_path = scenario_dir.join("official_row_injection.json");
        fs::write(
            &scenario_path,
            scenario_set
                .to_json_string()
                .map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;

        let max_symbols = row_injection
            .injected_rows
            .iter()
            .map(|row| row.symbol.to_ascii_uppercase())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            .max(1)
            .min(config.max_symbols);
        let reference_config = CommitteeReferencePackConfig {
            reference_pack_id: format!("{}-official-reference-pack", config.replication_id),
            scenario_set_paths: vec![scenario_path.display().to_string()],
            candle_series_paths: candle_series_paths.to_vec(),
            output_root: config.output_dir().display().to_string(),
            max_rows: row_injection
                .injected_rows
                .len()
                .max(1)
                .min(config.max_rows),
            max_symbols,
            allow_estimated_references: true,
            allow_controlled_fixture_references: config.allow_controlled_fixture,
            allow_yfinance_research: config.allow_yfinance_research,
            allow_fixture: config.allow_fixture || config.allow_controlled_fixture,
            reason_codes: vec![ReasonCode::OfficialReferenceReplicationBuilt],
            ..CommitteeReferencePackConfig::default()
        };
        let bundle =
            super::committee_reference_pack_runner::CommitteeReferencePackRunner::default()
                .run(&reference_config)?;
        let linked_pack = build_linked_pack_from_reference_pack(
            &bundle.reference_pack,
            format!("{}-linked", config.replication_id),
        );
        let report = report_from_bundle(config, &bundle, coverage_report);
        Ok(OfficialReferenceReplicationArtifacts {
            report,
            closure_report: bundle.sufficiency_closure_report.clone(),
            linked_pack: Some(linked_pack),
            bundle: Some(bundle),
        })
    }
}

impl OfficialReferenceReplicationReport {
    pub fn to_text(&self) -> String {
        [
            format!("replication_id={}", self.replication_id),
            format!("outcome_reference_count={}", self.outcome_reference_count),
            format!("baseline_reference_count={}", self.baseline_reference_count),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denied_counterfactual_count={}",
                self.risk_denied_counterfactual_count
            ),
            format!(
                "official_ready_reference_count={}",
                self.official_ready_reference_count
            ),
            format!(
                "controlled_reference_count={}",
                self.controlled_reference_count
            ),
            format!(
                "crypto_only_reference_count={}",
                self.crypto_only_reference_count
            ),
            format!(
                "research_only_reference_count={}",
                self.research_only_reference_count
            ),
            format!(
                "diagnostic_only_reference_count={}",
                self.diagnostic_only_reference_count
            ),
            format!("replication_status={:?}", self.replication_status),
        ]
        .join("\n")
    }
}

pub fn build_linked_pack_from_reference_pack(
    pack: &GeneratedCommitteeReferencePack,
    linker_id: String,
) -> OutcomeLinkedCommitteeScenarioPack {
    let mut linked_rows = Vec::new();
    let mut outcome_linked_count = 0usize;
    let mut baseline_linked_count = 0usize;
    let mut external_linked_count = 0usize;
    let mut no_trade_counterfactual_count = 0usize;
    let mut risk_denial_counterfactual_count = 0usize;
    let mut no_lookahead_violations = 0usize;
    for row in &pack.scenario_rows {
        let references = pack
            .generated_references
            .iter()
            .filter(|reference| reference.scenario_row_id == row.scenario_row_id)
            .filter(|reference| reference.built())
            .collect::<Vec<_>>();
        let outcome_reference = references
            .iter()
            .find(|reference| {
                reference.reference_kind == GeneratedReferenceKind::TripleBarrierOutcome
            })
            .and_then(|reference| reference.outcome_reference.clone());
        let baseline_reference = references
            .iter()
            .find(|reference| reference.reference_kind == GeneratedReferenceKind::BaselineAction)
            .and_then(|reference| reference.baseline_reference.clone());
        let external_reference = references
            .iter()
            .find(|reference| {
                reference.reference_kind == GeneratedReferenceKind::ExternalPredictionAction
            })
            .and_then(|reference| reference.external_reference.clone());
        if let Some(outcome) = &outcome_reference {
            if outcome.benchmark_eligible() {
                outcome_linked_count += 1;
            }
            if outcome.no_trade_counterfactual() {
                no_trade_counterfactual_count += 1;
            }
            if outcome.risk_denial_counterfactual() {
                risk_denial_counterfactual_count += 1;
            }
            if !outcome.no_lookahead_safe {
                no_lookahead_violations += 1;
            }
        }
        if baseline_reference.is_some() {
            baseline_linked_count += 1;
        }
        if external_reference.is_some() {
            external_linked_count += 1;
        }
        if outcome_reference.is_some()
            || baseline_reference.is_some()
            || external_reference.is_some()
        {
            linked_rows.push(OutcomeLinkedCommitteeScenarioRow {
                scenario_row: row.clone(),
                outcome_reference,
                baseline_reference,
                external_reference,
                reason_codes: stable_reason_codes(&[
                    ReasonCode::OfficialReferenceReplicationBuilt,
                    ReasonCode::CommitteeOutcomeReferenceBuilt,
                ]),
            });
        }
    }
    let pack_copy = pack.to_official_pack();
    let unmatched_rows = pack
        .scenario_rows
        .iter()
        .filter(|row| {
            !linked_rows
                .iter()
                .any(|linked| linked.scenario_row.scenario_row_id == row.scenario_row_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    OutcomeLinkedCommitteeScenarioPack {
        pack: pack_copy.clone(),
        linked_rows,
        unmatched_rows,
        link_summary: CommitteeOutcomeLinkSummary {
            linker_id,
            matched_rows: outcome_linked_count
                .max(baseline_linked_count)
                .max(external_linked_count),
            unmatched_rows: pack_copy.rows.len().saturating_sub(
                outcome_linked_count
                    .max(baseline_linked_count)
                    .max(external_linked_count),
            ),
            timestamp_tolerance_ms: 0,
            strict_timestamp_match: true,
            no_lookahead_violations,
            warnings: Vec::new(),
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialReferenceReplicationBuilt,
                ReasonCode::CommitteeOutcomeLinkerBuilt,
            ]),
        },
        outcome_linked_count,
        baseline_linked_count,
        external_linked_count,
        no_trade_counterfactual_count,
        risk_denial_counterfactual_count,
        no_lookahead_violations,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialReferenceReplicationBuilt,
            ReasonCode::CommitteeOutcomeLinkerBuilt,
        ]),
    }
}

fn report_from_bundle(
    config: &OfficialEvidenceReplicationConfig,
    bundle: &CommitteeReferencePackBundle,
    coverage_report: &OfficialCandleCoverageReport,
) -> OfficialReferenceReplicationReport {
    let counts = classify_reference_counts(&bundle.reference_pack);
    let quality = &bundle.quality_report;
    let replication_status = if counts.official_ready > 0 {
        OfficialReferenceReplicationStatus::OfficialReferencesGenerated
    } else if counts.controlled > 0 {
        OfficialReferenceReplicationStatus::ControlledReferencesOnly
    } else if counts.crypto > 0 {
        OfficialReferenceReplicationStatus::CryptoOnlyReferences
    } else if matches!(
        coverage_report.coverage_status,
        OfficialCandleCoverageStatus::MissingOfficialCandles
    ) {
        OfficialReferenceReplicationStatus::MissingOfficialCandleData
    } else if quality.outcome_reference_count == 0 {
        OfficialReferenceReplicationStatus::MissingOutcomeWindows
    } else if matches!(
        quality.quality_status,
        ReferencePackQualityStatus::NeedMoreBaselineReferences
    ) {
        OfficialReferenceReplicationStatus::MissingBaselineReferences
    } else if matches!(
        quality.quality_status,
        ReferencePackQualityStatus::NeedMoreNoTradeCounterfactuals
            | ReferencePackQualityStatus::NeedMoreRiskDeniedCounterfactuals
    ) {
        OfficialReferenceReplicationStatus::MissingCounterfactualDepth
    } else {
        OfficialReferenceReplicationStatus::ReferenceGenerationBlocked
    };
    OfficialReferenceReplicationReport {
        replication_id: config.replication_id.clone(),
        generated_reference_pack: Some(bundle.reference_pack.clone()),
        reference_pack_quality: Some(bundle.quality_report.clone()),
        outcome_reference_count: bundle.reference_pack.generated_outcome_count,
        baseline_reference_count: bundle.reference_pack.generated_baseline_count,
        no_trade_counterfactual_count: bundle.reference_pack.generated_no_trade_count,
        risk_denied_counterfactual_count: bundle.reference_pack.generated_risk_denied_count,
        official_ready_reference_count: counts.official_ready,
        controlled_reference_count: counts.controlled,
        crypto_only_reference_count: counts.crypto,
        research_only_reference_count: counts.research,
        diagnostic_only_reference_count: bundle.reference_pack.diagnostic_only_count,
        replication_status,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialReferenceReplicationBuilt,
            ReasonCode::CommitteeReferencePackBuilt,
        ]),
    }
}

fn blocked_report(
    config: &OfficialEvidenceReplicationConfig,
    status: OfficialReferenceReplicationStatus,
) -> OfficialReferenceReplicationArtifacts {
    OfficialReferenceReplicationArtifacts {
        report: OfficialReferenceReplicationReport {
            replication_id: config.replication_id.clone(),
            generated_reference_pack: None,
            reference_pack_quality: None,
            outcome_reference_count: 0,
            baseline_reference_count: 0,
            no_trade_counterfactual_count: 0,
            risk_denied_counterfactual_count: 0,
            official_ready_reference_count: 0,
            controlled_reference_count: 0,
            crypto_only_reference_count: 0,
            research_only_reference_count: 0,
            diagnostic_only_reference_count: 0,
            replication_status: status,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialReferenceReplicationBuilt,
                ReasonCode::EvidenceStillInsufficient,
            ]),
        },
        bundle: None,
        linked_pack: None,
        closure_report: None,
    }
}

struct ReferenceBoundaryCounts {
    official_ready: usize,
    controlled: usize,
    crypto: usize,
    research: usize,
}

fn classify_reference_counts(pack: &GeneratedCommitteeReferencePack) -> ReferenceBoundaryCounts {
    let mut counts = ReferenceBoundaryCounts {
        official_ready: 0,
        controlled: 0,
        crypto: 0,
        research: 0,
    };
    for reference in pack
        .generated_references
        .iter()
        .filter(|reference| reference.built())
    {
        let Some(row) = pack.row_for_reference(reference) else {
            continue;
        };
        match classify_row_boundary(row) {
            OfficialEvidenceBoundary::OfficialNonCrypto => {
                if reference.reference_kind != GeneratedReferenceKind::ExternalPredictionAction {
                    counts.official_ready += 1;
                }
            }
            OfficialEvidenceBoundary::OfficialCryptoOnly => counts.crypto += 1,
            OfficialEvidenceBoundary::Controlled => counts.controlled += 1,
            OfficialEvidenceBoundary::ResearchOnly => counts.research += 1,
            OfficialEvidenceBoundary::FixtureOnly | OfficialEvidenceBoundary::Unknown => {}
        }
    }
    counts
}
