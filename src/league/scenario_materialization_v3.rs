use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::comparable_evidence_builder::ComparableEvidenceBuilder;
use super::official_candle_coverage_pack::{
    OfficialCandleSeriesDescriptor, load_pack_from_path_or_config, normalize_symbol,
};
use super::official_committee_pack::{
    OfficialCommitteeScenarioPack, OfficialCommitteeScenarioPackConfig,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryConfig, OfficialReadyRowInventoryReport,
    OfficialReadyRowInventoryRunner,
};
use super::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow,
    OfficialCommitteeScenarioPackBuilder,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioMaterializationV3Config {
    pub materialization_id: String,
    #[serde(default)]
    pub official_ready_inventory_path: Option<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_coverage_pack_paths: Vec<String>,
    #[serde(default)]
    pub official_scenario_pack_paths: Vec<String>,
    #[serde(default)]
    pub canonical_csv_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub prefer_existing_scenario_rows: bool,
    #[serde(default = "default_true")]
    pub allow_canonical_csv_projection: bool,
    #[serde(default)]
    pub require_feature_summary: bool,
    #[serde(default = "default_true")]
    pub allow_limited_feature_summary: bool,
    #[serde(default = "default_true")]
    pub require_official_ready_match: bool,
    #[serde(default = "default_true")]
    pub require_preflight: bool,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScenarioMaterializationV3Level {
    ExistingRowLevelScenario,
    OfficialReadyCandleProjected,
    CanonicalCsvProjected,
    LimitedFeatureProjected,
    SummaryDerivedDiagnostic,
    #[default]
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioMaterializationV3Record {
    pub row_id: String,
    pub scenario_row_id: String,
    pub materialization_level: ScenarioMaterializationV3Level,
    pub official_ready_match_used: bool,
    #[serde(default)]
    pub candle_series_id: Option<String>,
    pub feature_summary_available: bool,
    pub limited_feature_summary: bool,
    pub source_class: ComparableEvidenceSourceClass,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScenarioMaterializationV3Status {
    MaterializationImproved,
    OfficialRowLevelMaterialized,
    LimitedFeatureMaterialized,
    StillMissingScenarioRows,
    StillSummaryDerivedOnly,
    SourceIneligible,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioMaterializationV3Report {
    pub records: Vec<ScenarioMaterializationV3Record>,
    pub materialized_count: usize,
    pub row_level_count: usize,
    pub limited_feature_count: usize,
    pub rejected_count: usize,
    pub official_materialized_count: usize,
    pub diagnostic_only_count: usize,
    pub materialization_status: ScenarioMaterializationV3Status,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScenarioMaterializationV3Runner;

impl Default for ScenarioMaterializationV3Config {
    fn default() -> Self {
        Self {
            materialization_id: "scenario-materialization-v3".to_string(),
            official_ready_inventory_path: None,
            comparable_evidence_bundle_paths: Vec::new(),
            official_candle_coverage_pack_paths: Vec::new(),
            official_scenario_pack_paths: Vec::new(),
            canonical_csv_paths: Vec::new(),
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            prefer_existing_scenario_rows: true,
            allow_canonical_csv_projection: true,
            require_feature_summary: false,
            allow_limited_feature_summary: true,
            require_official_ready_match: true,
            require_preflight: true,
            require_provenance: true,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ScenarioMaterializationV3Config {
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
        if self.materialization_id.trim().is_empty() {
            return Err("scenario materialization v3 id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("scenario materialization v3 paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "scenario materialization v3 max_rows must be between 1 and 500".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "scenario materialization v3 max_symbols must be between 1 and 5".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "scenario materialization v3 max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.materialization_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.official_ready_inventory_path
            .iter()
            .cloned()
            .chain(self.comparable_evidence_bundle_paths.iter().cloned())
            .chain(self.official_candle_coverage_pack_paths.iter().cloned())
            .chain(self.official_scenario_pack_paths.iter().cloned())
            .chain(self.canonical_csv_paths.iter().cloned())
            .collect()
    }
}

impl ScenarioMaterializationV3Runner {
    pub fn run(
        &self,
        config: &ScenarioMaterializationV3Config,
    ) -> Result<ScenarioMaterializationV3Report, String> {
        config.validate()?;
        let inventory = load_inventory(config)?;
        let descriptors = load_descriptors(config)?;
        let scenario_rows = load_scenario_rows(config)?;
        self.run_from_inventory(config, &inventory, &descriptors, &scenario_rows)
    }

    pub fn run_from_inventory(
        &self,
        config: &ScenarioMaterializationV3Config,
        inventory: &OfficialReadyRowInventoryReport,
        descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
        scenario_rows: &BTreeMap<String, CommitteeScenarioRow>,
    ) -> Result<ScenarioMaterializationV3Report, String> {
        config.validate()?;
        if inventory.items.len() > config.max_rows {
            return Err(format!(
                "scenario materialization v3 loaded {} rows which exceeds max_rows {}",
                inventory.items.len(),
                config.max_rows
            ));
        }
        let unique_symbols = inventory
            .items
            .iter()
            .map(|item| item.symbol.clone())
            .collect::<BTreeSet<_>>();
        if unique_symbols.len() > config.max_symbols {
            return Err(format!(
                "scenario materialization v3 loaded {} symbols which exceeds max_symbols {}",
                unique_symbols.len(),
                config.max_symbols
            ));
        }
        let storage_bytes = input_storage_bytes(&config.all_paths());
        if storage_bytes > config.max_bytes {
            return Err(format!(
                "scenario materialization v3 input size {} exceeds max_bytes {}",
                storage_bytes, config.max_bytes
            ));
        }
        let mut records = inventory
            .items
            .iter()
            .map(|item| build_record(config, item, descriptors, scenario_rows))
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.row_id
                .cmp(&right.row_id)
                .then(left.scenario_row_id.cmp(&right.scenario_row_id))
                .then(left.materialization_level.cmp(&right.materialization_level))
        });
        let materialized_count = records
            .iter()
            .filter(|record| {
                record.materialization_level != ScenarioMaterializationV3Level::Rejected
            })
            .count();
        let row_level_count = records
            .iter()
            .filter(|record| {
                matches!(
                    record.materialization_level,
                    ScenarioMaterializationV3Level::ExistingRowLevelScenario
                        | ScenarioMaterializationV3Level::OfficialReadyCandleProjected
                        | ScenarioMaterializationV3Level::CanonicalCsvProjected
                )
            })
            .count();
        let limited_feature_count = records
            .iter()
            .filter(|record| record.limited_feature_summary)
            .count();
        let rejected_count = records
            .iter()
            .filter(|record| {
                record.materialization_level == ScenarioMaterializationV3Level::Rejected
            })
            .count();
        let official_materialized_count = records
            .iter()
            .filter(|record| {
                record.materialization_level != ScenarioMaterializationV3Level::Rejected
                    && matches!(
                        record.source_class,
                        ComparableEvidenceSourceClass::OfficialNonCrypto
                            | ComparableEvidenceSourceClass::OfficialCryptoOnly
                    )
            })
            .count();
        let diagnostic_only_count = records
            .iter()
            .filter(|record| record.diagnostic_only)
            .count();
        let materialization_status = determine_status(
            &records,
            materialized_count,
            row_level_count,
            limited_feature_count,
            rejected_count,
            official_materialized_count,
            diagnostic_only_count,
        );
        Ok(ScenarioMaterializationV3Report {
            records,
            materialized_count,
            row_level_count,
            limited_feature_count,
            rejected_count,
            official_materialized_count,
            diagnostic_only_count,
            materialization_status,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl ScenarioMaterializationV3Report {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_default())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("materialized_count={}", self.materialized_count),
            format!("row_level_count={}", self.row_level_count),
            format!("limited_feature_count={}", self.limited_feature_count),
            format!("rejected_count={}", self.rejected_count),
            format!(
                "official_materialized_count={}",
                self.official_materialized_count
            ),
            format!("diagnostic_only_count={}", self.diagnostic_only_count),
            format!("materialization_status={:?}", self.materialization_status),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.records.iter().map(|record| {
            format!(
                "row_id={};scenario_row_id={};level={:?};official_ready_match_used={};candle_series_id={};feature_summary_available={};limited_feature_summary={};source_class={:?};diagnostic_only={}",
                record.row_id,
                record.scenario_row_id,
                record.materialization_level,
                record.official_ready_match_used,
                record.candle_series_id.clone().unwrap_or_default(),
                record.feature_summary_available,
                record.limited_feature_summary,
                record.source_class,
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
            output_dir.join("scenario_materialization_v3.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("scenario_materialization_v3.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn build_record(
    config: &ScenarioMaterializationV3Config,
    item: &super::official_ready_row_inventory::OfficialReadyRowInventoryItem,
    descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
    scenario_rows: &BTreeMap<String, CommitteeScenarioRow>,
) -> ScenarioMaterializationV3Record {
    let scenario_row = item
        .scenario_row_id
        .as_ref()
        .and_then(|id| scenario_rows.get(id));
    let descriptor = item
        .candle_series_id
        .as_ref()
        .and_then(|id| descriptors.get(id))
        .or_else(|| match_descriptor(item, descriptors));
    let mut reason_codes = item.reason_codes.clone();
    let generated_scenario_row_id = item.scenario_row_id.clone().unwrap_or_else(|| {
        format!(
            "smv3-{}",
            stable_hash_string(&format!(
                "{}:{}:{}",
                item.row_id, item.symbol, item.timestamp_ms
            ))
        )
    });
    if config.require_official_ready_match && !item.official_ready_match {
        reason_codes.push(ReasonCode::MissingOfficialCandles);
        return rejected_record(item, generated_scenario_row_id, reason_codes);
    }
    if config.require_no_lookahead_safe && !item.no_lookahead_safe {
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
        return rejected_record(item, generated_scenario_row_id, reason_codes);
    }
    if config.prefer_existing_scenario_rows {
        if item.has_scenario_row && item.scenario_row_id.is_some() && scenario_row.is_none() {
            let feature_summary_available = item.row_level && !item.summary_derived;
            if !config.require_feature_summary || feature_summary_available {
                return ScenarioMaterializationV3Record {
                    row_id: item.row_id.clone(),
                    scenario_row_id: item.scenario_row_id.clone().unwrap_or(generated_scenario_row_id),
                    materialization_level: ScenarioMaterializationV3Level::ExistingRowLevelScenario,
                    official_ready_match_used: item.official_ready_match,
                    candle_series_id: item.candle_series_id.clone(),
                    feature_summary_available,
                    limited_feature_summary: !feature_summary_available,
                    source_class: item.source_class,
                    diagnostic_only: item
                        .completeness_statuses
                        .contains(&super::official_ready_row_inventory::OfficialReadyRowCompletenessStatus::DiagnosticOnly),
                    reason_codes: stable_reason_codes(&reason_codes),
                };
            }
        }
        if let Some(existing) = scenario_row {
            let feature_summary_available = scenario_feature_available(existing);
            if !config.require_feature_summary || feature_summary_available {
                return ScenarioMaterializationV3Record {
                    row_id: item.row_id.clone(),
                    scenario_row_id: existing.scenario_row_id.clone(),
                    materialization_level: ScenarioMaterializationV3Level::ExistingRowLevelScenario,
                    official_ready_match_used: item.official_ready_match,
                    candle_series_id: item.candle_series_id.clone(),
                    feature_summary_available,
                    limited_feature_summary: false,
                    source_class: item.source_class,
                    diagnostic_only: item
                        .completeness_statuses
                        .contains(&super::official_ready_row_inventory::OfficialReadyRowCompletenessStatus::DiagnosticOnly),
                    reason_codes: stable_reason_codes(&reason_codes),
                };
            }
        }
    }
    let source_diagnostic = matches!(
        item.source_class,
        ComparableEvidenceSourceClass::ControlledDiagnostic
            | ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
    );
    let Some(descriptor) = descriptor else {
        reason_codes.push(ReasonCode::MissingRealLocalData);
        return if item.summary_derived {
            ScenarioMaterializationV3Record {
                row_id: item.row_id.clone(),
                scenario_row_id: generated_scenario_row_id,
                materialization_level: ScenarioMaterializationV3Level::SummaryDerivedDiagnostic,
                official_ready_match_used: item.official_ready_match,
                candle_series_id: item.candle_series_id.clone(),
                feature_summary_available: false,
                limited_feature_summary: true,
                source_class: item.source_class,
                diagnostic_only: true,
                reason_codes: stable_reason_codes(&reason_codes),
            }
        } else {
            rejected_record(item, generated_scenario_row_id, reason_codes)
        };
    };
    if config.require_preflight && !descriptor.preflight_ready {
        reason_codes.push(ReasonCode::MissingOfficialPreflight);
        return rejected_record(item, generated_scenario_row_id, reason_codes);
    }
    if config.require_provenance && !descriptor.provenance_available {
        reason_codes.push(ReasonCode::MissingOfficialProvenance);
        return rejected_record(item, generated_scenario_row_id, reason_codes);
    }
    let canonical_match = config.allow_canonical_csv_projection
        && config
            .canonical_csv_paths
            .iter()
            .any(|path| path == &descriptor.path);
    let limited_feature_summary = !item.row_level || item.summary_derived || source_diagnostic;
    let feature_summary_available =
        !limited_feature_summary || config.allow_limited_feature_summary;
    let materialization_level = if canonical_match {
        ScenarioMaterializationV3Level::CanonicalCsvProjected
    } else if !limited_feature_summary {
        ScenarioMaterializationV3Level::OfficialReadyCandleProjected
    } else if config.allow_limited_feature_summary {
        reason_codes.push(ReasonCode::FeatureUnavailable);
        ScenarioMaterializationV3Level::LimitedFeatureProjected
    } else {
        ScenarioMaterializationV3Level::SummaryDerivedDiagnostic
    };
    ScenarioMaterializationV3Record {
        row_id: item.row_id.clone(),
        scenario_row_id: generated_scenario_row_id,
        materialization_level,
        official_ready_match_used: item.official_ready_match,
        candle_series_id: Some(descriptor.candle_series_id.clone()),
        feature_summary_available,
        limited_feature_summary,
        source_class: item.source_class,
        diagnostic_only: source_diagnostic
            || item
                .completeness_statuses
                .contains(&super::official_ready_row_inventory::OfficialReadyRowCompletenessStatus::DiagnosticOnly)
            || materialization_level == ScenarioMaterializationV3Level::SummaryDerivedDiagnostic,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn rejected_record(
    item: &super::official_ready_row_inventory::OfficialReadyRowInventoryItem,
    scenario_row_id: String,
    reason_codes: Vec<ReasonCode>,
) -> ScenarioMaterializationV3Record {
    ScenarioMaterializationV3Record {
        row_id: item.row_id.clone(),
        scenario_row_id,
        materialization_level: ScenarioMaterializationV3Level::Rejected,
        official_ready_match_used: item.official_ready_match,
        candle_series_id: item.candle_series_id.clone(),
        feature_summary_available: false,
        limited_feature_summary: false,
        source_class: item.source_class,
        diagnostic_only: true,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn determine_status(
    records: &[ScenarioMaterializationV3Record],
    materialized_count: usize,
    row_level_count: usize,
    limited_feature_count: usize,
    rejected_count: usize,
    official_materialized_count: usize,
    diagnostic_only_count: usize,
) -> ScenarioMaterializationV3Status {
    if records.is_empty() {
        return ScenarioMaterializationV3Status::StillMissingScenarioRows;
    }
    if materialized_count == 0 && rejected_count == records.len() {
        if records.iter().all(|record| {
            matches!(
                record.source_class,
                ComparableEvidenceSourceClass::ControlledDiagnostic
                    | ComparableEvidenceSourceClass::YFinanceResearch
                    | ComparableEvidenceSourceClass::FixtureArchitectureTest
                    | ComparableEvidenceSourceClass::SyntheticTest
            )
        }) {
            return ScenarioMaterializationV3Status::SourceIneligible;
        }
        return ScenarioMaterializationV3Status::StillMissingScenarioRows;
    }
    if row_level_count > 0 && official_materialized_count > 0 {
        return ScenarioMaterializationV3Status::OfficialRowLevelMaterialized;
    }
    if limited_feature_count > 0 && materialized_count > 0 {
        return ScenarioMaterializationV3Status::LimitedFeatureMaterialized;
    }
    if diagnostic_only_count == records.len() {
        return ScenarioMaterializationV3Status::DiagnosticOnly;
    }
    if records.iter().any(|record| {
        record.materialization_level == ScenarioMaterializationV3Level::SummaryDerivedDiagnostic
    }) {
        return ScenarioMaterializationV3Status::StillSummaryDerivedOnly;
    }
    ScenarioMaterializationV3Status::MaterializationImproved
}

fn load_inventory(
    config: &ScenarioMaterializationV3Config,
) -> Result<OfficialReadyRowInventoryReport, String> {
    if let Some(path) = config.official_ready_inventory_path.as_deref() {
        if path.ends_with(".toml") {
            let inventory_config =
                OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
            OfficialReadyRowInventoryRunner::default().run(&inventory_config)
        } else {
            OfficialReadyRowInventoryReport::from_json_path(Path::new(path))
        }
    } else {
        let rows = load_rows(config)?;
        let inventory_config = OfficialReadyRowInventoryConfig {
            inventory_id: format!("{}-inventory", config.materialization_id),
            comparable_evidence_bundle_paths: config.comparable_evidence_bundle_paths.clone(),
            official_candle_coverage_pack_paths: config.official_candle_coverage_pack_paths.clone(),
            scenario_pack_paths: config.official_scenario_pack_paths.clone(),
            output_root: config.output_root.clone(),
            require_official_ready_match: config.require_official_ready_match,
            require_no_lookahead_safe: config.require_no_lookahead_safe,
            allow_controlled_diagnostic: true,
            allow_crypto_only: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            reason_codes: config.reason_codes.clone(),
            ..OfficialReadyRowInventoryConfig::default()
        };
        let descriptors = load_descriptors(config)?;
        let scenario_rows = load_scenario_rows(config)?;
        OfficialReadyRowInventoryRunner::default().run_from_rows(
            &inventory_config,
            &rows,
            &scenario_rows,
            &descriptors,
        )
    }
}

fn load_rows(
    config: &ScenarioMaterializationV3Config,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let mut rows = Vec::new();
    for path in &config.comparable_evidence_bundle_paths {
        let bundle = if path.ends_with(".toml") {
            let comparable_config =
                ComparableCommitteeEvidenceConfig::from_toml_path(Path::new(path))?;
            ComparableEvidenceBuilder::default().build(&comparable_config)?
        } else {
            ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path))?
        };
        rows.extend(bundle.rows);
    }
    Ok(rows)
}

fn load_descriptors(
    config: &ScenarioMaterializationV3Config,
) -> Result<BTreeMap<String, OfficialCandleSeriesDescriptor>, String> {
    let mut descriptors = BTreeMap::new();
    for path in &config.official_candle_coverage_pack_paths {
        let pack = load_pack_from_path_or_config(path)?;
        for descriptor in pack.descriptors {
            descriptors.insert(descriptor.candle_series_id.clone(), descriptor);
        }
    }
    Ok(descriptors)
}

fn load_scenario_rows(
    config: &ScenarioMaterializationV3Config,
) -> Result<BTreeMap<String, CommitteeScenarioRow>, String> {
    let mut rows = BTreeMap::new();
    for path in &config.official_scenario_pack_paths {
        let pack = if path.ends_with(".toml") {
            OfficialCommitteeScenarioPackBuilder::default().build(
                &OfficialCommitteeScenarioPackConfig::from_toml_path(Path::new(path))?,
            )?
        } else {
            OfficialCommitteeScenarioPack::from_json_path(Path::new(path))?
        };
        for row in pack.rows {
            rows.insert(row.scenario_row_id.clone(), row);
        }
    }
    Ok(rows)
}

fn match_descriptor<'a>(
    item: &super::official_ready_row_inventory::OfficialReadyRowInventoryItem,
    descriptors: &'a BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> Option<&'a OfficialCandleSeriesDescriptor> {
    let normalized = normalize_symbol(&item.symbol);
    descriptors.values().find(|descriptor| {
        descriptor.normalized_symbol == normalized
            && descriptor.timeframe.eq_ignore_ascii_case(&item.timeframe)
    })
}

fn scenario_feature_available(row: &CommitteeScenarioRow) -> bool {
    row.materialization_level == CommitteeScenarioMaterializationLevel::RowLevel
        || row.feature_vector.is_some()
        || !row.signal_summary.trim().is_empty()
}

fn input_storage_bytes(paths: &[String]) -> usize {
    paths
        .iter()
        .filter_map(|path| fs::metadata(path).ok())
        .map(|metadata| metadata.len() as usize)
        .sum()
}

fn default_output_root() -> String {
    "target/soma_scenario_materialization_v3".to_string()
}

fn default_max_rows() -> usize {
    500
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
