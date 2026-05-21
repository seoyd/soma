use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ablation::AblationStudyReport;
use super::before_after::{
    Sprint14BeforeAfterReport, Sprint14ComparableSummary, after_summary_from_decision,
    build_before_after_report,
};
use super::decision_router::{
    Sprint14DecisionRecord, Sprint14DecisionRouter, Sprint14EvidenceInput, Sprint14Track,
};
use super::evidence_gap::{EvidenceGapReport, build_evidence_gap_report};
use super::render::{sprint14_report_to_markdown, sprint14_report_to_text};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14TestSummary {
    pub local_only: bool,
    pub no_runtime_llm: bool,
    pub no_live_api: bool,
    pub no_real_broker: bool,
    pub no_real_order_execution: bool,
    pub no_new_personas: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14RiskReview {
    pub default_action_no_trade_preserved: bool,
    pub risk_governor_absolute_veto_preserved: bool,
    pub no_lookahead_changed: bool,
    pub no_live_mutation: bool,
    pub no_persona_expansion: bool,
    pub findings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sprint14TrackSpecificReport {
    NeedMoreExperiments(EvidenceGapReport),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14Report {
    pub decision_record: Sprint14DecisionRecord,
    pub before_after_report: Sprint14BeforeAfterReport,
    pub track_specific_report: Sprint14TrackSpecificReport,
    pub test_summary: Sprint14TestSummary,
    pub risk_review: Sprint14RiskReview,
    pub next_recommendation: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint14Report {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sprint14_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sprint14_report.txt"),
            sprint14_report_to_text(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sprint14_report.md"),
            sprint14_report_to_markdown(self),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sprint14Runner {
    pub decision_router: Sprint14DecisionRouter,
}

impl Sprint14Runner {
    pub fn run_from_ablation_report(
        &self,
        report: &AblationStudyReport,
        source_report_path: Option<String>,
    ) -> Sprint14Report {
        let evidence_inputs =
            Sprint14EvidenceInput::from_ablation_report(report, source_report_path);
        let decision_record = self.decision_router.decide(Some(&evidence_inputs));
        let before_summary = Sprint14ComparableSummary::from_ablation_report(report);
        let after_summary = after_summary_from_decision(&before_summary, &decision_record);
        let before_after_report = build_before_after_report(before_summary, after_summary);
        let track_specific_report = match decision_record.selected_track {
            Sprint14Track::NeedMoreExperiments => Sprint14TrackSpecificReport::NeedMoreExperiments(
                build_evidence_gap_report(&decision_record.evidence_inputs),
            ),
            _ => Sprint14TrackSpecificReport::NeedMoreExperiments(build_evidence_gap_report(
                &decision_record.evidence_inputs,
            )),
        };
        Sprint14Report {
            decision_record,
            before_after_report,
            track_specific_report,
            test_summary: Sprint14TestSummary {
                local_only: true,
                no_runtime_llm: true,
                no_live_api: true,
                no_real_broker: true,
                no_real_order_execution: true,
                no_new_personas: true,
                notes: vec!["selected track only; runtime trading logic unchanged".to_string()],
            },
            risk_review: Sprint14RiskReview {
                default_action_no_trade_preserved: true,
                risk_governor_absolute_veto_preserved: true,
                no_lookahead_changed: true,
                no_live_mutation: true,
                no_persona_expansion: true,
                findings: vec![
                    "Sprint 14 selected NeedMoreExperiments from non-comparable ablation evidence"
                        .to_string(),
                    "No runtime trading, broker, network, or persona path was changed".to_string(),
                ],
            },
            next_recommendation:
                "Expand local evidence first, then rerun Sprint 13 ablation before any new improvement track".to_string(),
            reason_codes: vec![ReasonCode::Sprint14DecisionBuilt, ReasonCode::Sprint14BeforeAfterBuilt],
        }
    }

    pub fn run_from_ablation_report_path(
        &self,
        ablation_report_path: &Path,
    ) -> Result<Sprint14Report, String> {
        if ablation_report_path.to_string_lossy().contains("://") {
            return Err("sprint14 report path must be local".to_string());
        }
        let report = AblationStudyReport::from_json_path(ablation_report_path)?;
        Ok(
            self.run_from_ablation_report(
                &report,
                Some(ablation_report_path.display().to_string()),
            ),
        )
    }
}
