use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::ReasonCode;
use crate::data::EvidenceSourceKind;

use super::committee_outcome_reference::{
    CommitteeBaselineAction, CommitteeBaselineReference, CommitteeExternalReference,
    CommitteeOutcomeReference, parse_evidence_source_kind, parse_triple_barrier_label,
};
use super::committee_scenario_loader::{CommitteeScenarioRow, CommitteeScenarioSet};
use super::official_committee_pack::OfficialCommitteeScenarioPack;
use super::persona_card_lite::PersonaHorizon;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcomeLinkerConfig {
    pub linker_id: String,
    #[serde(default)]
    pub scenario_pack_path: Option<String>,
    #[serde(default)]
    pub outcome_artifact_paths: Vec<String>,
    #[serde(default)]
    pub baseline_artifact_paths: Vec<String>,
    #[serde(default)]
    pub external_prediction_paths: Vec<String>,
    pub output_root: String,
    #[serde(default)]
    pub strict_timestamp_match: bool,
    #[serde(default = "default_max_timestamp_tolerance_ms")]
    pub max_timestamp_tolerance_ms: u64,
    #[serde(default = "default_true")]
    pub require_same_symbol: bool,
    #[serde(default = "default_true")]
    pub require_same_horizon: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcomeLinkSummary {
    pub linker_id: String,
    pub matched_rows: usize,
    pub unmatched_rows: usize,
    pub timestamp_tolerance_ms: u64,
    pub strict_timestamp_match: bool,
    pub no_lookahead_violations: usize,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeLinkedCommitteeScenarioRow {
    pub scenario_row: CommitteeScenarioRow,
    #[serde(default)]
    pub outcome_reference: Option<CommitteeOutcomeReference>,
    #[serde(default)]
    pub baseline_reference: Option<CommitteeBaselineReference>,
    #[serde(default)]
    pub external_reference: Option<CommitteeExternalReference>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeLinkedCommitteeScenarioPack {
    pub pack: OfficialCommitteeScenarioPack,
    pub linked_rows: Vec<OutcomeLinkedCommitteeScenarioRow>,
    pub unmatched_rows: Vec<CommitteeScenarioRow>,
    pub link_summary: CommitteeOutcomeLinkSummary,
    pub outcome_linked_count: usize,
    pub baseline_linked_count: usize,
    pub external_linked_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denial_counterfactual_count: usize,
    pub no_lookahead_violations: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeOutcomeLinker;

#[derive(Clone, Debug)]
struct OutcomeCandidate {
    reference: CommitteeOutcomeReference,
}

#[derive(Clone, Debug)]
struct BaselineCandidate {
    symbol: String,
    timestamp_ms: u64,
    horizon_bars: Option<usize>,
    reference: CommitteeBaselineReference,
}

#[derive(Clone, Debug)]
struct ExternalCandidate {
    symbol: String,
    timestamp_ms: u64,
    horizon_bars: Option<usize>,
    reference: CommitteeExternalReference,
}

impl Default for CommitteeOutcomeLinkerConfig {
    fn default() -> Self {
        Self {
            linker_id: "committee_outcome_linker".to_string(),
            scenario_pack_path: None,
            outcome_artifact_paths: Vec::new(),
            baseline_artifact_paths: Vec::new(),
            external_prediction_paths: Vec::new(),
            output_root: "target/soma_committee_outcome_linker".to_string(),
            strict_timestamp_match: false,
            max_timestamp_tolerance_ms: default_max_timestamp_tolerance_ms(),
            require_same_symbol: true,
            require_same_horizon: true,
            reason_codes: vec![ReasonCode::CommitteeOutcomeLinkerBuilt],
        }
    }
}

impl CommitteeOutcomeLinkerConfig {
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

    pub fn validate(&self) -> Result<(), String> {
        let mut path_iter = self
            .scenario_pack_path
            .iter()
            .cloned()
            .chain(self.outcome_artifact_paths.iter().cloned())
            .chain(self.baseline_artifact_paths.iter().cloned())
            .chain(self.external_prediction_paths.iter().cloned())
            .chain(std::iter::once(self.output_root.clone()));
        if path_iter.any(|path| path.contains("://")) {
            return Err("committee outcome linker paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.linker_id)
    }
}

impl CommitteeOutcomeLinker {
    pub fn link_from_config(
        &self,
        config: &CommitteeOutcomeLinkerConfig,
    ) -> Result<OutcomeLinkedCommitteeScenarioPack, String> {
        config.validate()?;
        let pack_path = config
            .scenario_pack_path
            .as_deref()
            .ok_or_else(|| "committee outcome linker requires scenario_pack_path".to_string())?;
        let pack = OfficialCommitteeScenarioPack::from_json_path(Path::new(pack_path))?;
        self.link(&pack, config)
    }

    pub fn link(
        &self,
        pack: &OfficialCommitteeScenarioPack,
        config: &CommitteeOutcomeLinkerConfig,
    ) -> Result<OutcomeLinkedCommitteeScenarioPack, String> {
        config.validate()?;
        let outcome_candidates = load_outcome_candidates(&config.outcome_artifact_paths)?;
        let baseline_candidates = load_baseline_candidates(&config.baseline_artifact_paths)?;
        let external_candidates = load_external_candidates(&config.external_prediction_paths)?;
        let mut linked_rows = Vec::new();
        let mut unmatched_rows = Vec::new();
        let mut outcome_linked_count = 0usize;
        let mut baseline_linked_count = 0usize;
        let mut external_linked_count = 0usize;
        let mut no_trade_counterfactual_count = 0usize;
        let mut risk_denial_counterfactual_count = 0usize;
        let mut no_lookahead_violations = 0usize;

        for row in &pack.rows {
            let outcome_reference = match_outcome(row, &outcome_candidates, config);
            let baseline_reference =
                match_baseline(row, &baseline_candidates, config).or_else(|| {
                    row.baseline_signal_summary
                        .as_deref()
                        .map(fallback_baseline_reference)
                });
            let external_reference = match_external(row, &external_candidates, config)
                .filter(|reference| reference.prediction_schema_valid);
            if let Some(reference) = &outcome_reference {
                if reference.benchmark_eligible() {
                    outcome_linked_count += 1;
                }
                if reference.no_trade_counterfactual() {
                    no_trade_counterfactual_count += 1;
                }
                if reference.risk_denial_counterfactual() {
                    risk_denial_counterfactual_count += 1;
                }
                if !reference.no_lookahead_safe {
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
                    reason_codes: vec![
                        ReasonCode::CommitteeOutcomeLinkerBuilt,
                        ReasonCode::CommitteeOutcomeReferenceBuilt,
                    ],
                });
            } else {
                unmatched_rows.push(row.clone());
            }
        }

        let warnings = if external_candidates.is_empty() {
            vec!["no external prediction references linked".to_string()]
        } else {
            Vec::new()
        };
        Ok(OutcomeLinkedCommitteeScenarioPack {
            pack: pack.clone(),
            linked_rows,
            unmatched_rows,
            link_summary: CommitteeOutcomeLinkSummary {
                linker_id: config.linker_id.clone(),
                matched_rows: outcome_linked_count
                    .max(baseline_linked_count)
                    .max(external_linked_count),
                unmatched_rows: pack.rows.len().saturating_sub(
                    outcome_linked_count
                        .max(baseline_linked_count)
                        .max(external_linked_count),
                ),
                timestamp_tolerance_ms: config.max_timestamp_tolerance_ms,
                strict_timestamp_match: config.strict_timestamp_match,
                no_lookahead_violations,
                warnings,
                reason_codes: vec![ReasonCode::CommitteeOutcomeLinkerBuilt],
            },
            outcome_linked_count,
            baseline_linked_count,
            external_linked_count,
            no_trade_counterfactual_count,
            risk_denial_counterfactual_count,
            no_lookahead_violations,
            reason_codes: config
                .reason_codes
                .iter()
                .cloned()
                .chain([
                    ReasonCode::CommitteeOutcomeLinkerBuilt,
                    ReasonCode::CommitteeOutcomeLinked,
                ])
                .collect(),
        })
    }
}

impl OutcomeLinkedCommitteeScenarioRow {
    pub fn benchmark_eligible(&self) -> bool {
        self.outcome_reference
            .as_ref()
            .is_some_and(CommitteeOutcomeReference::benchmark_eligible)
    }

    pub fn to_benchmark_row(&self) -> CommitteeScenarioRow {
        let mut row = self.scenario_row.clone();
        if let Some(reference) = &self.outcome_reference {
            row.outcome_reference = Some(reference.outcome_id.clone());
            if reference.no_trade_counterfactual() {
                row.no_trade_counterfactual = Some(reference.outcome_id.clone());
            }
            if reference.risk_denial_counterfactual() {
                row.risk_denial_counterfactual = Some(reference.outcome_id.clone());
            }
        }
        if let Some(reference) = &self.baseline_reference {
            row.baseline_signal_summary =
                Some(reference.baseline_action.as_summary_str().to_string());
        }
        if let Some(reference) = &self.external_reference {
            row.external_prediction_summary = reference.external_action.clone();
        }
        row
    }
}

impl OutcomeLinkedCommitteeScenarioPack {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_benchmark_scenario_set(&self, scenario_id: &str) -> CommitteeScenarioSet {
        let rows = if self.outcome_linked_count > 0 {
            self.linked_rows
                .iter()
                .filter(|row| row.benchmark_eligible())
                .map(OutcomeLinkedCommitteeScenarioRow::to_benchmark_row)
                .collect::<Vec<_>>()
        } else {
            self.pack.rows.clone()
        };
        CommitteeScenarioSet {
            scenario_id: scenario_id.to_string(),
            row_count: rows.len(),
            official_row_count: rows
                .iter()
                .filter(|row| row.evidence_source_kind.readiness_eligible())
                .count(),
            research_only_row_count: rows
                .iter()
                .filter(|row| row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch)
                .count(),
            fixture_row_count: rows
                .iter()
                .filter(|row| {
                    matches!(
                        row.source_kind,
                        super::committee_scenario_loader::CommitteeScenarioSourceKind::Fixture
                            | super::committee_scenario_loader::CommitteeScenarioSourceKind::SyntheticTest
                    )
                })
                .count(),
            skipped_row_count: self.pack.rows.len().saturating_sub(rows.len()),
            source_summary: format!("{}|OutcomeLinked", self.pack.source_summary),
            rows,
            reason_codes: self.reason_codes.clone(),
        }
    }

    pub fn to_text(&self) -> String {
        [
            self.link_summary.to_text(),
            format!("outcome_linked_count={}", self.outcome_linked_count),
            format!("baseline_linked_count={}", self.baseline_linked_count),
            format!("external_linked_count={}", self.external_linked_count),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denial_counterfactual_count={}",
                self.risk_denial_counterfactual_count
            ),
            format!("no_lookahead_violations={}", self.no_lookahead_violations),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("outcome_link_summary.txt"),
            self.link_summary.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("outcome_linked_pack.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl CommitteeOutcomeLinkSummary {
    pub fn to_text(&self) -> String {
        [
            format!("linker_id={}", self.linker_id),
            format!("matched_rows={}", self.matched_rows),
            format!("unmatched_rows={}", self.unmatched_rows),
            format!("timestamp_tolerance_ms={}", self.timestamp_tolerance_ms),
            format!("strict_timestamp_match={}", self.strict_timestamp_match),
            format!("no_lookahead_violations={}", self.no_lookahead_violations),
            format!("warnings={}", self.warnings.join("|")),
        ]
        .join("\n")
    }
}

fn load_outcome_candidates(paths: &[String]) -> Result<Vec<OutcomeCandidate>, String> {
    let mut candidates = Vec::new();
    for value in load_values(paths, &["outcomes", "rows", "records", "references"])? {
        let symbol = value
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| "outcome reference is missing symbol".to_string())?;
        let timestamp_ms = value
            .get("timestamp_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| "outcome reference is missing timestamp_ms".to_string())?;
        let horizon_bars = value
            .get("horizon_bars")
            .and_then(Value::as_u64)
            .unwrap_or(24) as usize;
        let outcome_id = value
            .get("outcome_id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!("{}-{timestamp_ms}-{horizon_bars}", normalize_symbol(symbol))
            });
        candidates.push(OutcomeCandidate {
            reference: CommitteeOutcomeReference {
                outcome_id,
                decision_id: value
                    .get("decision_id")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                symbol: normalize_symbol(symbol),
                timestamp_ms,
                horizon_bars,
                triple_barrier_label: parse_triple_barrier_label(
                    value.get("triple_barrier_label").and_then(Value::as_str),
                ),
                net_return_pct: value.get("net_return_pct").and_then(Value::as_f64),
                max_favorable_excursion_pct: value
                    .get("max_favorable_excursion_pct")
                    .and_then(Value::as_f64),
                max_adverse_excursion_pct: value
                    .get("max_adverse_excursion_pct")
                    .and_then(Value::as_f64),
                cost_bps: value
                    .get("cost_bps")
                    .and_then(Value::as_f64)
                    .unwrap_or_default(),
                slippage_bps: value
                    .get("slippage_bps")
                    .and_then(Value::as_f64)
                    .unwrap_or_default(),
                source_kind: parse_evidence_source_kind(
                    value.get("source_kind").and_then(Value::as_str),
                ),
                no_lookahead_safe: value
                    .get("no_lookahead_safe")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                reason_codes: vec![
                    ReasonCode::CommitteeOutcomeReferenceBuilt,
                    ReasonCode::CostApplied,
                ],
            },
        });
    }
    candidates.sort_by(|left, right| {
        left.reference
            .symbol
            .cmp(&right.reference.symbol)
            .then(
                left.reference
                    .timestamp_ms
                    .cmp(&right.reference.timestamp_ms),
            )
            .then(
                left.reference
                    .horizon_bars
                    .cmp(&right.reference.horizon_bars),
            )
            .then(left.reference.outcome_id.cmp(&right.reference.outcome_id))
    });
    Ok(candidates)
}

fn load_baseline_candidates(paths: &[String]) -> Result<Vec<BaselineCandidate>, String> {
    let mut candidates = Vec::new();
    for value in load_values(
        paths,
        &["baselines", "baseline_references", "rows", "records"],
    )? {
        let symbol = value
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| "baseline reference is missing symbol".to_string())?;
        let timestamp_ms = value
            .get("timestamp_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| "baseline reference is missing timestamp_ms".to_string())?;
        let action = value
            .get("baseline_action")
            .and_then(Value::as_str)
            .map(CommitteeBaselineAction::from_summary)
            .unwrap_or_default();
        candidates.push(BaselineCandidate {
            symbol: normalize_symbol(symbol),
            timestamp_ms,
            horizon_bars: value
                .get("horizon_bars")
                .and_then(Value::as_u64)
                .map(|value| value as usize),
            reference: CommitteeBaselineReference {
                baseline_action: action,
                baseline_confidence: value.get("baseline_confidence").and_then(Value::as_f64),
                baseline_expected_edge: value.get("baseline_expected_edge").and_then(Value::as_f64),
                baseline_reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
                reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
            },
        });
    }
    candidates.sort_by(|left, right| {
        left.symbol
            .cmp(&right.symbol)
            .then(left.timestamp_ms.cmp(&right.timestamp_ms))
            .then(left.horizon_bars.cmp(&right.horizon_bars))
            .then(
                left.reference
                    .baseline_action
                    .cmp(&right.reference.baseline_action),
            )
    });
    Ok(candidates)
}

fn load_external_candidates(paths: &[String]) -> Result<Vec<ExternalCandidate>, String> {
    let mut candidates = Vec::new();
    for value in load_values(
        paths,
        &["external_predictions", "predictions", "rows", "records"],
    )? {
        let symbol = value
            .get("symbol")
            .and_then(Value::as_str)
            .ok_or_else(|| "external prediction reference is missing symbol".to_string())?;
        let timestamp_ms = value
            .get("timestamp_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| "external prediction reference is missing timestamp_ms".to_string())?;
        let external_action = value
            .get("external_action")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                value
                    .get("action")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });
        let external_p_win = value
            .get("external_p_win")
            .and_then(Value::as_f64)
            .or_else(|| value.get("p_win").and_then(Value::as_f64));
        let external_confidence = value
            .get("external_confidence")
            .and_then(Value::as_f64)
            .or_else(|| value.get("confidence").and_then(Value::as_f64));
        let prediction_schema_valid = external_action.is_some()
            || external_p_win
                .map(|value| (0.0..=1.0).contains(&value))
                .unwrap_or(false)
            || external_confidence
                .map(|value| (0.0..=1.0).contains(&value))
                .unwrap_or(false);
        candidates.push(ExternalCandidate {
            symbol: normalize_symbol(symbol),
            timestamp_ms,
            horizon_bars: value
                .get("horizon_bars")
                .and_then(Value::as_u64)
                .map(|value| value as usize),
            reference: CommitteeExternalReference {
                external_action,
                external_p_win,
                external_confidence,
                prediction_schema_valid,
                reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
            },
        });
    }
    candidates.sort_by(|left, right| {
        left.symbol
            .cmp(&right.symbol)
            .then(left.timestamp_ms.cmp(&right.timestamp_ms))
            .then(left.horizon_bars.cmp(&right.horizon_bars))
    });
    Ok(candidates)
}

fn match_outcome(
    row: &CommitteeScenarioRow,
    candidates: &[OutcomeCandidate],
    config: &CommitteeOutcomeLinkerConfig,
) -> Option<CommitteeOutcomeReference> {
    let row_symbol = normalize_symbol(&row.symbol);
    let row_horizon = horizon_bars(row.target_horizon);
    candidates
        .iter()
        .filter(|candidate| {
            candidate_matches(
                &row_symbol,
                row.timestamp_ms,
                row_horizon,
                &candidate.reference.symbol,
                candidate.reference.timestamp_ms,
                Some(candidate.reference.horizon_bars),
                config,
            )
        })
        .min_by_key(|candidate| {
            timestamp_distance(row.timestamp_ms, candidate.reference.timestamp_ms)
        })
        .map(|candidate| candidate.reference.clone())
}

fn match_baseline(
    row: &CommitteeScenarioRow,
    candidates: &[BaselineCandidate],
    config: &CommitteeOutcomeLinkerConfig,
) -> Option<CommitteeBaselineReference> {
    let row_symbol = normalize_symbol(&row.symbol);
    let row_horizon = horizon_bars(row.target_horizon);
    candidates
        .iter()
        .filter(|candidate| {
            candidate_matches(
                &row_symbol,
                row.timestamp_ms,
                row_horizon,
                &candidate.symbol,
                candidate.timestamp_ms,
                candidate.horizon_bars,
                config,
            )
        })
        .min_by_key(|candidate| timestamp_distance(row.timestamp_ms, candidate.timestamp_ms))
        .map(|candidate| candidate.reference.clone())
}

fn match_external(
    row: &CommitteeScenarioRow,
    candidates: &[ExternalCandidate],
    config: &CommitteeOutcomeLinkerConfig,
) -> Option<CommitteeExternalReference> {
    let row_symbol = normalize_symbol(&row.symbol);
    let row_horizon = horizon_bars(row.target_horizon);
    candidates
        .iter()
        .filter(|candidate| {
            candidate_matches(
                &row_symbol,
                row.timestamp_ms,
                row_horizon,
                &candidate.symbol,
                candidate.timestamp_ms,
                candidate.horizon_bars,
                config,
            )
        })
        .min_by_key(|candidate| timestamp_distance(row.timestamp_ms, candidate.timestamp_ms))
        .map(|candidate| candidate.reference.clone())
}

fn candidate_matches(
    row_symbol: &str,
    row_timestamp_ms: u64,
    row_horizon_bars: usize,
    candidate_symbol: &str,
    candidate_timestamp_ms: u64,
    candidate_horizon_bars: Option<usize>,
    config: &CommitteeOutcomeLinkerConfig,
) -> bool {
    if config.require_same_symbol && row_symbol != candidate_symbol {
        return false;
    }
    if config.require_same_horizon
        && candidate_horizon_bars.is_some_and(|horizon| horizon != row_horizon_bars)
    {
        return false;
    }
    let distance = timestamp_distance(row_timestamp_ms, candidate_timestamp_ms);
    if config.strict_timestamp_match {
        distance == 0
    } else {
        distance <= config.max_timestamp_tolerance_ms
    }
}

fn fallback_baseline_reference(summary: &str) -> CommitteeBaselineReference {
    CommitteeBaselineReference {
        baseline_action: CommitteeBaselineAction::from_summary(summary),
        baseline_confidence: None,
        baseline_expected_edge: None,
        baseline_reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
        reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
    }
}

fn load_values(paths: &[String], keys: &[&str]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        if let Some(items) = parsed.as_array() {
            values.extend(items.iter().cloned());
            continue;
        }
        if let Some(items) = keys
            .iter()
            .find_map(|key| parsed.get(key).and_then(Value::as_array))
        {
            values.extend(items.iter().cloned());
            continue;
        }
        values.push(parsed);
    }
    Ok(values)
}

fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_ascii_uppercase()
}

fn timestamp_distance(left: u64, right: u64) -> u64 {
    left.max(right) - left.min(right)
}

fn horizon_bars(horizon: PersonaHorizon) -> usize {
    match horizon {
        PersonaHorizon::Intraday => 6,
        PersonaHorizon::Swing => 24,
        PersonaHorizon::MultiDay => 48,
        PersonaHorizon::LongTerm => 96,
    }
}

fn default_true() -> bool {
    true
}

fn default_max_timestamp_tolerance_ms() -> u64 {
    300_000
}
