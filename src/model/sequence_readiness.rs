use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};

fn default_output_root() -> String {
    "target/sprint55/sequence_readiness".to_string()
}

fn default_target_window_lengths() -> Vec<usize> {
    vec![32, 64, 96]
}

fn default_target_horizons() -> Vec<usize> {
    vec![4, 8, 16]
}

fn default_min_sequence_windows() -> usize {
    256
}

fn default_min_symbols() -> usize {
    4
}

fn default_min_outcome_diversity() -> usize {
    3
}

fn default_max_summary_derived_ratio() -> f64 {
    0.25
}

fn default_max_storage_bytes() -> usize {
    32 * 1024 * 1024
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceDatasetReadinessConfig {
    pub readiness_id: String,
    #[serde(default)]
    pub official_evidence_scaleout_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_diversity_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_bundle_paths: Vec<String>,
    #[serde(default)]
    pub feature_schema_paths: Vec<String>,
    #[serde(default)]
    pub candle_pack_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_target_window_lengths")]
    pub target_window_lengths: Vec<usize>,
    #[serde(default = "default_target_horizons")]
    pub target_horizons: Vec<usize>,
    #[serde(default = "default_min_sequence_windows")]
    pub min_sequence_windows: usize,
    #[serde(default = "default_min_symbols")]
    pub min_symbols: usize,
    #[serde(default = "default_min_outcome_diversity")]
    pub min_outcome_diversity: usize,
    #[serde(default = "default_max_summary_derived_ratio")]
    pub max_summary_derived_ratio: f64,
    #[serde(default = "default_max_storage_bytes")]
    pub max_storage_bytes: usize,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default = "default_true")]
    pub require_official_non_crypto: bool,
    #[serde(default)]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_research_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for SequenceDatasetReadinessConfig {
    fn default() -> Self {
        Self {
            readiness_id: "sprint55_sequence_readiness".to_string(),
            official_evidence_scaleout_paths: Vec::new(),
            official_evidence_diversity_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            complete_row_bundle_paths: Vec::new(),
            feature_schema_paths: Vec::new(),
            candle_pack_paths: Vec::new(),
            output_root: default_output_root(),
            target_window_lengths: default_target_window_lengths(),
            target_horizons: default_target_horizons(),
            min_sequence_windows: default_min_sequence_windows(),
            min_symbols: default_min_symbols(),
            min_outcome_diversity: default_min_outcome_diversity(),
            max_summary_derived_ratio: default_max_summary_derived_ratio(),
            max_storage_bytes: default_max_storage_bytes(),
            require_no_lookahead_safe: true,
            require_official_non_crypto: true,
            allow_crypto_only: false,
            allow_research_only: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl SequenceDatasetReadinessConfig {
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
        if self.readiness_id.trim().is_empty() {
            return Err("sequence readiness id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("sequence-readiness config paths must be local".to_string());
        }
        if self.target_window_lengths.is_empty() {
            return Err("sequence-readiness target_window_lengths must not be empty".to_string());
        }
        if self.target_horizons.is_empty() {
            return Err("sequence-readiness target_horizons must not be empty".to_string());
        }
        if self.min_sequence_windows == 0 || self.min_symbols == 0 || self.max_storage_bytes == 0 {
            return Err("sequence-readiness numeric thresholds must be positive".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.readiness_id)
    }

    pub fn all_input_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .official_evidence_scaleout_paths
                .iter()
                .chain(self.official_evidence_diversity_paths.iter())
                .chain(self.comparable_evidence_bundle_paths.iter())
                .chain(self.complete_row_bundle_paths.iter())
                .chain(self.feature_schema_paths.iter())
                .chain(self.candle_pack_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceDatasetReadinessStatus {
    ReadyForSequenceDatasetExport,
    NeedMoreRows,
    NeedMoreSymbols,
    NeedMoreOutcomeLabels,
    NeedFeatureSchemaLock,
    NeedNoLookaheadProof,
    NeedStorageBudget,
    ResearchOnly,
    DiagnosticOnly,
    NotReady,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetReadinessReport {
    pub readiness_id: String,
    pub row_count: usize,
    pub official_row_count: usize,
    pub complete_row_count: usize,
    pub estimated_sequence_windows: usize,
    pub symbols: Vec<String>,
    pub horizons: Vec<usize>,
    pub window_lengths: Vec<usize>,
    pub outcome_label_distribution: BTreeMap<String, usize>,
    pub feature_schema_locked: bool,
    pub no_lookahead_safe: bool,
    pub storage_estimate_bytes: usize,
    pub readiness_status: SequenceDatasetReadinessStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl SequenceDatasetReadinessReport {
    pub fn from_value(value: &Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok().or_else(|| {
            value
                .get("sequence_readiness_report")
                .and_then(|item| serde_json::from_value(item.clone()).ok())
        })
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("readiness_id={}", self.readiness_id),
            format!("row_count={}", self.row_count),
            format!("official_row_count={}", self.official_row_count),
            format!("complete_row_count={}", self.complete_row_count),
            format!(
                "estimated_sequence_windows={}",
                self.estimated_sequence_windows
            ),
            format!("symbols={}", self.symbols.join("|")),
            format!(
                "horizons={}",
                self.horizons
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!(
                "window_lengths={}",
                self.window_lengths
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!(
                "outcome_label_distribution={}",
                self.outcome_label_distribution
                    .iter()
                    .map(|(label, count)| format!("{label}:{count}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("feature_schema_locked={}", self.feature_schema_locked),
            format!("no_lookahead_safe={}", self.no_lookahead_safe),
            format!("storage_estimate_bytes={}", self.storage_estimate_bytes),
            format!("readiness_status={:?}", self.readiness_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sequence_dataset_readiness_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sequence_dataset_readiness_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SequenceDatasetReadinessRunner;

impl SequenceDatasetReadinessRunner {
    pub fn run(
        &self,
        config: &SequenceDatasetReadinessConfig,
    ) -> Result<SequenceDatasetReadinessReport, String> {
        config.validate()?;
        let mut warnings = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        let values = load_values(&config.all_input_paths(), &mut warnings, &mut reason_codes);

        let mut row_count = 0usize;
        let mut official_row_count = 0usize;
        let mut complete_row_count = 0usize;
        let mut estimated_sequence_windows = 0usize;
        let mut symbols = BTreeSet::new();
        let mut horizons = BTreeSet::new();
        let mut window_lengths = BTreeSet::new();
        let mut outcome_label_distribution = BTreeMap::<String, usize>::new();
        let mut feature_schema_locked = false;
        let mut no_lookahead_safe = false;
        let mut storage_estimate_bytes = 0usize;
        let mut research_only = false;
        let mut crypto_only = false;
        let mut summary_derived_ratio = 0.0f64;

        for value in &values {
            row_count = row_count.max(usize_field(value, &["row_count", "rows", "total_rows"]));
            official_row_count = official_row_count.max(usize_field(
                value,
                &[
                    "official_row_count",
                    "official_rows",
                    "official_non_crypto_row_count",
                ],
            ));
            complete_row_count = complete_row_count.max(usize_field(
                value,
                &[
                    "complete_row_count",
                    "complete_rows",
                    "official_complete_rows",
                ],
            ));
            estimated_sequence_windows = estimated_sequence_windows.max(usize_field(
                value,
                &[
                    "estimated_sequence_windows",
                    "sequence_windows",
                    "estimated_windows",
                ],
            ));
            storage_estimate_bytes = storage_estimate_bytes.max(usize_field(
                value,
                &[
                    "storage_estimate_bytes",
                    "estimated_bytes",
                    "sequence_storage_bytes",
                ],
            ));
            feature_schema_locked |= bool_field(
                value,
                &[
                    "feature_schema_locked",
                    "feature_schema_lock",
                    "feature_schema_stable",
                ],
            )
            .unwrap_or(false);
            no_lookahead_safe |= bool_field(
                value,
                &["no_lookahead_safe", "no_lookahead_proved", "leakage_safe"],
            )
            .unwrap_or(false);
            summary_derived_ratio = summary_derived_ratio.max(
                f64_field(
                    value,
                    &["max_summary_derived_ratio", "summary_derived_ratio"],
                )
                .unwrap_or(0.0),
            );
            research_only |= source_matches(value, &["yfinance", "research_only", "research-only"]);
            crypto_only |= source_matches(value, &["crypto_only", "crypto-only"]);

            symbols.extend(string_list_field(
                value,
                &["symbols", "symbol_ids", "symbol"],
            ));
            horizons.extend(usize_list_field(
                value,
                &["horizons", "target_horizons", "horizon"],
            ));
            window_lengths.extend(usize_list_field(
                value,
                &["window_lengths", "target_window_lengths", "window_size"],
            ));
            merge_outcome_distribution(
                &mut outcome_label_distribution,
                &object_field_usize(value, &["outcome_label_distribution", "label_distribution"]),
            );
        }

        if horizons.is_empty() {
            horizons.extend(config.target_horizons.iter().copied());
        }
        if window_lengths.is_empty() {
            window_lengths.extend(config.target_window_lengths.iter().copied());
        }
        if estimated_sequence_windows == 0 {
            estimated_sequence_windows = estimate_sequence_windows(row_count, &window_lengths);
        }
        if storage_estimate_bytes == 0 {
            storage_estimate_bytes = estimated_sequence_windows
                .saturating_mul(window_lengths.iter().max().copied().unwrap_or(1))
                .saturating_mul(16);
        }

        let label_diversity = outcome_label_distribution
            .iter()
            .filter(|(_, count)| **count > 0)
            .count();
        let min_rows_for_windows = config
            .min_sequence_windows
            .saturating_add(window_lengths.iter().max().copied().unwrap_or(1))
            .saturating_sub(1);

        let mut blockers = Vec::new();
        let readiness_status = if research_only && config.allow_research_only {
            blockers.push("source boundary remains research-only; official non-crypto evidence is still required".to_string());
            SequenceDatasetReadinessStatus::ResearchOnly
        } else if crypto_only && !config.allow_crypto_only {
            blockers.push(
                "crypto-only evidence remains diagnostic-only for this sequence export gate"
                    .to_string(),
            );
            SequenceDatasetReadinessStatus::DiagnosticOnly
        } else if config.require_official_non_crypto && official_row_count == 0 {
            blockers.push("official non-crypto rows are still missing".to_string());
            SequenceDatasetReadinessStatus::NotReady
        } else if !feature_schema_locked {
            blockers.push("feature schema lock is required before sequence export".to_string());
            SequenceDatasetReadinessStatus::NeedFeatureSchemaLock
        } else if config.require_no_lookahead_safe && !no_lookahead_safe {
            blockers.push("no-lookahead proof is required before sequence export".to_string());
            SequenceDatasetReadinessStatus::NeedNoLookaheadProof
        } else if storage_estimate_bytes > config.max_storage_bytes {
            blockers.push("sequence storage estimate exceeds the configured budget".to_string());
            SequenceDatasetReadinessStatus::NeedStorageBudget
        } else if row_count < min_rows_for_windows
            || estimated_sequence_windows < config.min_sequence_windows
        {
            blockers.push("more rows and windows are required before sequence export".to_string());
            SequenceDatasetReadinessStatus::NeedMoreRows
        } else if symbols.len() < config.min_symbols {
            blockers.push("more symbol coverage is required before sequence export".to_string());
            SequenceDatasetReadinessStatus::NeedMoreSymbols
        } else if label_diversity < config.min_outcome_diversity {
            blockers.push(
                "more outcome-label diversity is required before sequence export".to_string(),
            );
            SequenceDatasetReadinessStatus::NeedMoreOutcomeLabels
        } else {
            SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport
        };

        if summary_derived_ratio > config.max_summary_derived_ratio {
            warnings.push(
                "summary-derived rows are too prominent; keep source boundaries explicit"
                    .to_string(),
            );
        }
        if matches!(
            readiness_status,
            SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport
        ) && official_row_count < complete_row_count
        {
            warnings.push("complete rows exceed explicitly official rows; verify manifests before benchmark promotion".to_string());
        }
        warnings.push(
            "sequence readiness does not imply profitability or live trading readiness".to_string(),
        );

        let mut codes = vec![ReasonCode::SequenceDatasetSpecBuilt];
        if feature_schema_locked {
            codes.push(ReasonCode::FeatureSchemaValidated);
        } else {
            codes.push(ReasonCode::FeatureSchemaMismatch);
        }
        if storage_estimate_bytes > config.max_storage_bytes {
            codes.push(ReasonCode::SequenceStorageBudgetExceeded);
        }
        reason_codes.extend(codes);

        let report = SequenceDatasetReadinessReport {
            readiness_id: config.readiness_id.clone(),
            row_count,
            official_row_count,
            complete_row_count,
            estimated_sequence_windows,
            symbols: symbols.into_iter().collect(),
            horizons: horizons.into_iter().collect(),
            window_lengths: window_lengths.into_iter().collect(),
            outcome_label_distribution,
            feature_schema_locked,
            no_lookahead_safe,
            storage_estimate_bytes,
            readiness_status,
            blockers: stable_ordered_strings(&blockers),
            warnings: stable_ordered_strings(&warnings),
            reason_codes: stable_reason_codes(&reason_codes),
        };
        report.write_to_dir(&config.output_dir())?;
        Ok(report)
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
                    warnings.push(format!("failed to parse sequence readiness input: {err}"));
                    reason_codes.push(ReasonCode::DataLoadFailed);
                }
            },
            Err(_) => {
                warnings.push(format!("missing sequence readiness input: {path}"));
                reason_codes.push(ReasonCode::MissingFile);
            }
        }
    }
    values
}

fn estimate_sequence_windows(row_count: usize, window_lengths: &BTreeSet<usize>) -> usize {
    let Some(max_window) = window_lengths.iter().max().copied() else {
        return row_count;
    };
    row_count.saturating_sub(max_window).saturating_add(1)
}

fn merge_outcome_distribution(
    output: &mut BTreeMap<String, usize>,
    input: &BTreeMap<String, usize>,
) {
    for (label, count) in input {
        *output.entry(label.clone()).or_insert(0) += count;
    }
}

fn source_matches(value: &Value, tokens: &[&str]) -> bool {
    let mut matches = false;
    for key in [
        "source_class",
        "source_boundary",
        "data_source_class",
        "provider_label",
    ] {
        if let Some(text) = string_field(value, &[key]) {
            let lowered = text.to_ascii_lowercase();
            matches |= tokens.iter().any(|token| lowered.contains(token));
        }
    }
    matches
}

fn bool_field(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|key| {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        matches.into_iter().find_map(|item| match item {
            Value::Bool(flag) => Some(*flag),
            Value::String(text) => match text.to_ascii_lowercase().as_str() {
                "true" | "ready" | "locked" => Some(true),
                "false" | "blocked" | "missing" => Some(false),
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
        .filter_map(as_usize)
        .max()
        .unwrap_or(0)
}

fn f64_field(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        matches.into_iter().find_map(|item| match item {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse::<f64>().ok(),
            _ => None,
        })
    })
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

fn string_list_field(value: &Value, keys: &[&str]) -> Vec<String> {
    let mut items = BTreeSet::new();
    for key in keys {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        for item in matches {
            match item {
                Value::Array(values) => {
                    for value in values {
                        if let Some(text) = value.as_str() {
                            items.insert(text.to_string());
                        }
                    }
                }
                Value::String(text) => {
                    items.insert(text.to_string());
                }
                _ => {}
            }
        }
    }
    items.into_iter().collect()
}

fn usize_list_field(value: &Value, keys: &[&str]) -> Vec<usize> {
    let mut items = BTreeSet::new();
    for key in keys {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        for item in matches {
            match item {
                Value::Array(values) => {
                    for value in values {
                        if let Some(number) = as_usize(value) {
                            items.insert(number);
                        }
                    }
                }
                _ => {
                    if let Some(number) = as_usize(item) {
                        items.insert(number);
                    }
                }
            }
        }
    }
    items.into_iter().collect()
}

fn object_field_usize(value: &Value, keys: &[&str]) -> BTreeMap<String, usize> {
    let mut output = BTreeMap::new();
    for key in keys {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        for item in matches {
            if let Value::Object(map) = item {
                for (label, value) in map {
                    if let Some(count) = as_usize(value) {
                        *output.entry(label.clone()).or_insert(0) += count;
                    }
                }
            }
        }
    }
    output
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

fn as_usize(value: &Value) -> Option<usize> {
    match value {
        Value::Number(number) => number.as_u64().map(|number| number as usize),
        Value::String(text) => text.parse::<usize>().ok(),
        _ => None,
    }
}
