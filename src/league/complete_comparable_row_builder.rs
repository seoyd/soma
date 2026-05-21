use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::baseline_reference_backfill::{BaselineBackfillSource, BaselineReferenceBackfillPlan};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::counterfactual_backfill_plan::CounterfactualBackfillPlan;
use super::outcome_reference_backfill::OutcomeReferenceBackfillPlan;
use super::scenario_materialization_v3::{
    ScenarioMaterializationV3Level, ScenarioMaterializationV3Record,
    ScenarioMaterializationV3Report,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteComparableRowBuilderConfig {
    pub bundle_id: String,
    #[serde(default)]
    pub allow_diagnostic_complete: bool,
    #[serde(default = "default_true")]
    pub allow_controlled_diagnostic: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompleteComparableRowBuildStatus {
    BuiltComplete,
    BuiltDiagnosticComplete,
    BuiltPartial,
    SkippedMissingScenario,
    SkippedMissingOutcome,
    SkippedMissingBaseline,
    SkippedMissingCounterfactuals,
    SkippedSourceIneligible,
    SkippedNoLookahead,
    #[default]
    SkippedInsufficientArtifacts,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteComparableRowBuildRecord {
    pub row_id: String,
    pub source_row_id: String,
    pub status: CompleteComparableRowBuildStatus,
    pub materialization_level: ScenarioMaterializationV3Level,
    pub outcome_backfilled: bool,
    pub baseline_backfilled: bool,
    pub no_trade_backfilled: bool,
    pub risk_denied_backfilled: bool,
    pub official_complete: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteComparableRowBundle {
    pub bundle_id: String,
    pub rows: Vec<ComparableCommitteeEvidenceRow>,
    pub build_records: Vec<CompleteComparableRowBuildRecord>,
    pub complete_rows: usize,
    pub official_complete_rows: usize,
    pub diagnostic_complete_rows: usize,
    pub partial_rows: usize,
    pub skipped_rows: usize,
    pub outcome_reference_count: usize,
    pub baseline_reference_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub storage_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompleteComparableRowBuilder;

impl Default for CompleteComparableRowBuilderConfig {
    fn default() -> Self {
        Self {
            bundle_id: "complete-comparable-row-bundle".to_string(),
            allow_diagnostic_complete: false,
            allow_controlled_diagnostic: true,
            allow_crypto_only: true,
            allow_yfinance_research: false,
            allow_fixture: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CompleteComparableRowBuilder {
    pub fn build(
        &self,
        config: &CompleteComparableRowBuilderConfig,
        rows: &[ComparableCommitteeEvidenceRow],
        materialization: &ScenarioMaterializationV3Report,
        outcome_plan: &OutcomeReferenceBackfillPlan,
        baseline_plan: &BaselineReferenceBackfillPlan,
        counterfactual_plan: &CounterfactualBackfillPlan,
    ) -> CompleteComparableRowBundle {
        let materialized = materialization_map(materialization);
        let outcome_items = outcome_map(outcome_plan);
        let baseline_items = baseline_map(baseline_plan);
        let counterfactual_items = counterfactual_map(counterfactual_plan);
        let mut rows_out = Vec::new();
        let mut build_records = Vec::new();
        for row in rows {
            let record = materialized.get(&row.row_id);
            let outcome_item = outcome_items.get(&row.row_id);
            let baseline_item = baseline_items.get(&row.row_id);
            let counterfactual_item = counterfactual_items.get(&row.row_id);
            let (updated, build_record) = apply_row_build(
                config,
                row,
                record,
                outcome_item,
                baseline_item,
                counterfactual_item,
            );
            rows_out.push(updated);
            build_records.push(build_record);
        }
        rows_out.sort_by(|left, right| {
            left.row_id
                .cmp(&right.row_id)
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timestamp_ms.cmp(&right.timestamp_ms))
        });
        build_records.sort_by(|left, right| left.row_id.cmp(&right.row_id));
        let complete_rows = build_records
            .iter()
            .filter(|record| {
                matches!(
                    record.status,
                    CompleteComparableRowBuildStatus::BuiltComplete
                        | CompleteComparableRowBuildStatus::BuiltDiagnosticComplete
                )
            })
            .count();
        let official_complete_rows = build_records
            .iter()
            .filter(|record| record.official_complete)
            .count();
        let diagnostic_complete_rows = build_records
            .iter()
            .filter(|record| {
                record.status == CompleteComparableRowBuildStatus::BuiltDiagnosticComplete
            })
            .count();
        let partial_rows = build_records
            .iter()
            .filter(|record| record.status == CompleteComparableRowBuildStatus::BuiltPartial)
            .count();
        let skipped_rows = build_records
            .len()
            .saturating_sub(complete_rows + partial_rows);
        let outcome_reference_count = rows_out
            .iter()
            .filter(|row| row.outcome_reference_available)
            .count();
        let baseline_reference_count = rows_out
            .iter()
            .filter(|row| row.baseline_reference_available)
            .count();
        let no_trade_counterfactual_count = rows_out
            .iter()
            .filter(|row| row.no_trade_counterfactual_available)
            .count();
        let risk_denied_counterfactual_count = rows_out
            .iter()
            .filter(|row| row.risk_denied_counterfactual_available)
            .count();
        let storage_bytes = serde_json::to_vec(&rows_out)
            .map(|bytes| bytes.len())
            .unwrap_or_default();
        CompleteComparableRowBundle {
            bundle_id: config.bundle_id.clone(),
            rows: rows_out,
            build_records,
            complete_rows,
            official_complete_rows,
            diagnostic_complete_rows,
            partial_rows,
            skipped_rows,
            outcome_reference_count,
            baseline_reference_count,
            no_trade_counterfactual_count,
            risk_denied_counterfactual_count,
            storage_bytes,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly])
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl CompleteComparableRowBundle {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.bundle_id.clone()))
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("bundle_id={}", self.bundle_id),
            format!("complete_rows={}", self.complete_rows),
            format!("official_complete_rows={}", self.official_complete_rows),
            format!("diagnostic_complete_rows={}", self.diagnostic_complete_rows),
            format!("partial_rows={}", self.partial_rows),
            format!("skipped_rows={}", self.skipped_rows),
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
            format!("storage_bytes={}", self.storage_bytes),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.build_records.iter().map(|record| {
            format!(
                "row_id={};status={:?};materialization_level={:?};outcome_backfilled={};baseline_backfilled={};no_trade_backfilled={};risk_denied_backfilled={};official_complete={};diagnostic_only={}",
                record.row_id,
                record.status,
                record.materialization_level,
                record.outcome_backfilled,
                record.baseline_backfilled,
                record.no_trade_backfilled,
                record.risk_denied_backfilled,
                record.official_complete,
                record.diagnostic_only,
            )
        }));
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("complete_comparable_rows.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("complete_comparable_row_bundle.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn apply_row_build(
    config: &CompleteComparableRowBuilderConfig,
    row: &ComparableCommitteeEvidenceRow,
    materialization: Option<&ScenarioMaterializationV3Record>,
    outcome_item: Option<&&super::outcome_reference_backfill::OutcomeReferenceBackfillPlanItem>,
    baseline_item: Option<&&super::baseline_reference_backfill::BaselineReferenceBackfillPlanItem>,
    counterfactual_item: Option<
        &&super::counterfactual_backfill_plan::CounterfactualBackfillPlanItem,
    >,
) -> (
    ComparableCommitteeEvidenceRow,
    CompleteComparableRowBuildRecord,
) {
    let mut updated = row.clone();
    let materialization_level = materialization
        .map(|record| record.materialization_level)
        .unwrap_or(ScenarioMaterializationV3Level::Rejected);
    let source_allowed = source_allowed(row.source_class, config);
    let scenario_ready = materialization.is_some_and(|record| {
        record.materialization_level != ScenarioMaterializationV3Level::Rejected
    }) || row.scenario_row_id.is_some();
    if let Some(record) = materialization {
        updated.scenario_row_id = Some(record.scenario_row_id.clone());
        updated.row_level = matches!(
            record.materialization_level,
            ScenarioMaterializationV3Level::ExistingRowLevelScenario
                | ScenarioMaterializationV3Level::OfficialReadyCandleProjected
                | ScenarioMaterializationV3Level::CanonicalCsvProjected
        );
        updated.summary_derived = matches!(
            record.materialization_level,
            ScenarioMaterializationV3Level::LimitedFeatureProjected
                | ScenarioMaterializationV3Level::SummaryDerivedDiagnostic
        );
    }
    let outcome_backfilled = outcome_item.is_some_and(|item| item.can_build_from_candles);
    if outcome_backfilled {
        updated.outcome_reference_available = true;
        updated
            .reason_codes
            .push(ReasonCode::CommitteeOutcomeReferenceBuilt);
    }
    let baseline_backfilled = baseline_item.is_some_and(|item| item.can_backfill);
    if let Some(item) = baseline_item {
        if item.can_backfill {
            updated.baseline_reference_available = true;
            if item.source == BaselineBackfillSource::DeterministicNoTradeBaseline {
                updated.baseline_action = Some(updated.no_trade_baseline_action.clone());
            }
            if item.diagnostic_only {
                updated.diagnostic_only = true;
            }
        }
    }
    let no_trade_backfilled = counterfactual_item.is_some_and(|item| item.can_build_no_trade);
    let risk_denied_backfilled = counterfactual_item.is_some_and(|item| item.can_build_risk_denied);
    if no_trade_backfilled {
        updated.no_trade_counterfactual_available = true;
        updated.reason_codes.push(ReasonCode::NoTradeCounterfactual);
    }
    if risk_denied_backfilled {
        updated.risk_denied_counterfactual_available = true;
        updated
            .reason_codes
            .push(ReasonCode::RiskDeniedCounterfactual);
    }
    let has_outcome = updated.outcome_reference_available;
    let has_baseline = updated.baseline_reference_available;
    let has_counterfactuals =
        updated.no_trade_counterfactual_available && updated.risk_denied_counterfactual_available;
    let diagnostic_only = updated.diagnostic_only
        || materialization.is_some_and(|record| record.diagnostic_only)
        || baseline_item.is_some_and(|item| item.diagnostic_only)
        || matches!(
            row.source_class,
            ComparableEvidenceSourceClass::ControlledDiagnostic
                | ComparableEvidenceSourceClass::YFinanceResearch
                | ComparableEvidenceSourceClass::FixtureArchitectureTest
                | ComparableEvidenceSourceClass::SyntheticTest
        );
    updated.reason_codes = stable_reason_codes(&updated.reason_codes);
    let status = if !source_allowed {
        CompleteComparableRowBuildStatus::SkippedSourceIneligible
    } else if !updated.no_lookahead_safe {
        CompleteComparableRowBuildStatus::SkippedNoLookahead
    } else if !scenario_ready {
        CompleteComparableRowBuildStatus::SkippedMissingScenario
    } else if !has_outcome {
        if baseline_backfilled || no_trade_backfilled || risk_denied_backfilled {
            CompleteComparableRowBuildStatus::BuiltPartial
        } else {
            CompleteComparableRowBuildStatus::SkippedMissingOutcome
        }
    } else if !has_baseline {
        if outcome_backfilled || no_trade_backfilled || risk_denied_backfilled {
            CompleteComparableRowBuildStatus::BuiltPartial
        } else {
            CompleteComparableRowBuildStatus::SkippedMissingBaseline
        }
    } else if !has_counterfactuals {
        CompleteComparableRowBuildStatus::SkippedMissingCounterfactuals
    } else if diagnostic_only {
        if config.allow_diagnostic_complete {
            CompleteComparableRowBuildStatus::BuiltDiagnosticComplete
        } else {
            CompleteComparableRowBuildStatus::BuiltPartial
        }
    } else {
        CompleteComparableRowBuildStatus::BuiltComplete
    };
    let official_complete = status == CompleteComparableRowBuildStatus::BuiltComplete
        && row.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto
        && !updated.summary_derived
        && updated.no_lookahead_safe;
    let build_record = CompleteComparableRowBuildRecord {
        row_id: updated.row_id.clone(),
        source_row_id: row.row_id.clone(),
        status,
        materialization_level,
        outcome_backfilled,
        baseline_backfilled,
        no_trade_backfilled,
        risk_denied_backfilled,
        official_complete,
        diagnostic_only,
        reason_codes: stable_reason_codes(&updated.reason_codes),
    };
    (updated, build_record)
}

fn materialization_map(
    report: &ScenarioMaterializationV3Report,
) -> BTreeMap<String, ScenarioMaterializationV3Record> {
    report
        .records
        .iter()
        .map(|record| (record.row_id.clone(), record.clone()))
        .collect()
}

fn outcome_map(
    plan: &OutcomeReferenceBackfillPlan,
) -> BTreeMap<String, &super::outcome_reference_backfill::OutcomeReferenceBackfillPlanItem> {
    plan.items
        .iter()
        .map(|item| (item.row_id.clone(), item))
        .collect()
}

fn baseline_map(
    plan: &BaselineReferenceBackfillPlan,
) -> BTreeMap<String, &super::baseline_reference_backfill::BaselineReferenceBackfillPlanItem> {
    plan.items
        .iter()
        .map(|item| (item.row_id.clone(), item))
        .collect()
}

fn counterfactual_map(
    plan: &CounterfactualBackfillPlan,
) -> BTreeMap<String, &super::counterfactual_backfill_plan::CounterfactualBackfillPlanItem> {
    plan.items
        .iter()
        .map(|item| (item.row_id.clone(), item))
        .collect()
}

fn source_allowed(
    source_class: ComparableEvidenceSourceClass,
    config: &CompleteComparableRowBuilderConfig,
) -> bool {
    match source_class {
        ComparableEvidenceSourceClass::OfficialNonCrypto => true,
        ComparableEvidenceSourceClass::OfficialCryptoOnly => config.allow_crypto_only,
        ComparableEvidenceSourceClass::ControlledDiagnostic => config.allow_controlled_diagnostic,
        ComparableEvidenceSourceClass::YFinanceResearch => config.allow_yfinance_research,
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => config.allow_fixture,
        ComparableEvidenceSourceClass::Unknown => false,
    }
}

fn default_true() -> bool {
    true
}
