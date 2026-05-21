use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::committee_artifact_resolver::CommitteeArtifactResolver;
use super::committee_materialization::{
    CommitteeMaterializationConfig, CommitteeScenarioMaterializerV2,
};
use super::committee_scenario_loader::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow, CommitteeScenarioSet,
    CommitteeScenarioSourceKind,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCommitteePackSourceKind {
    OfficialApiCollected,
    OfficialCryptoPublic,
    YFinanceResearch,
    Fixture,
    SyntheticTest,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCommitteeScenarioPackConfig {
    pub pack_id: String,
    #[serde(default)]
    pub input_artifact_paths: Vec<String>,
    #[serde(default = "default_allowed_source_kinds")]
    pub allowed_source_kinds: Vec<OfficialCommitteePackSourceKind>,
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default)]
    pub require_preflight: bool,
    #[serde(default)]
    pub require_outcome_reference: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub allow_summary_derived_rows: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCommitteeScenarioPack {
    pub pack_id: String,
    pub rows: Vec<CommitteeScenarioRow>,
    pub source_summary: String,
    pub official_row_count: usize,
    pub crypto_only_row_count: usize,
    pub yfinance_row_count: usize,
    pub fixture_row_count: usize,
    pub row_level_count: usize,
    pub summary_derived_count: usize,
    pub outcome_linked_count: usize,
    pub baseline_reference_count: usize,
    pub external_reference_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denial_counterfactual_count: usize,
    pub storage_bytes: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialCommitteeScenarioPackBuilder;

impl Default for OfficialCommitteeScenarioPackConfig {
    fn default() -> Self {
        Self {
            pack_id: "official_committee_pack".to_string(),
            input_artifact_paths: Vec::new(),
            allowed_source_kinds: default_allowed_source_kinds(),
            output_root: "target/soma_committee_official_pack".to_string(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_provenance: true,
            require_preflight: true,
            require_outcome_reference: false,
            allow_crypto_only: true,
            allow_yfinance_research: false,
            allow_fixture: false,
            allow_summary_derived_rows: false,
            reason_codes: vec![ReasonCode::OfficialCommitteePackBuilt],
        }
    }
}

impl OfficialCommitteeScenarioPackConfig {
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
        if is_remote_path(&self.output_root)
            || self
                .input_artifact_paths
                .iter()
                .any(|path| is_remote_path(path))
        {
            return Err("official committee pack paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err("official committee pack max_rows must be between 1 and 100".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("official committee pack max_symbols must be between 1 and 50".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official committee pack max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.pack_id)
    }

    pub fn to_materialization_config(&self) -> CommitteeMaterializationConfig {
        CommitteeMaterializationConfig {
            materialization_id: format!("{}-materialized", self.pack_id),
            input_artifact_paths: self.input_artifact_paths.clone(),
            output_root: self.output_root.clone(),
            max_rows: self.max_rows,
            max_symbols: self.max_symbols,
            max_bytes: self.max_bytes,
            allow_summary_derived_rows: self.allow_summary_derived_rows,
            prefer_row_level_artifacts: true,
            require_provenance: self.require_provenance,
            min_data_quality: 0.70,
            reason_codes: vec![
                ReasonCode::CommitteeMaterializationBuilt,
                ReasonCode::OfficialCommitteePackBuilt,
            ],
            ..CommitteeMaterializationConfig::default()
        }
    }
}

impl OfficialCommitteeScenarioPackBuilder {
    pub fn build(
        &self,
        config: &OfficialCommitteeScenarioPackConfig,
    ) -> Result<OfficialCommitteeScenarioPack, String> {
        config.validate()?;
        let resolver = CommitteeArtifactResolver::default();
        let descriptors = config
            .input_artifact_paths
            .iter()
            .map(|path| resolver.resolve(path))
            .collect::<Vec<_>>();
        let has_any_preflight = descriptors
            .iter()
            .any(|descriptor| descriptor.preflight_available);
        let storage_bytes = config
            .input_artifact_paths
            .iter()
            .filter_map(|path| fs::metadata(path).ok())
            .map(|metadata| metadata.len() as usize)
            .sum::<usize>()
            .min(config.max_bytes);
        let materialized = CommitteeScenarioMaterializerV2::default()
            .materialize(&config.to_materialization_config())?;
        let mut rows = materialized
            .rows
            .into_iter()
            .filter(|row| row_allowed(row, config, has_any_preflight))
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
        let counts = counts_for_rows(&rows);
        let official_row_count = counts
            .get(&OfficialCommitteePackSourceKind::OfficialApiCollected)
            .copied()
            .unwrap_or(0)
            + counts
                .get(&OfficialCommitteePackSourceKind::OfficialCryptoPublic)
                .copied()
                .unwrap_or(0);
        let crypto_only_row_count = rows
            .iter()
            .filter(|row| row.market == ProviderMarket::Crypto)
            .count();
        let yfinance_row_count = counts
            .get(&OfficialCommitteePackSourceKind::YFinanceResearch)
            .copied()
            .unwrap_or(0);
        let fixture_row_count = counts
            .get(&OfficialCommitteePackSourceKind::Fixture)
            .copied()
            .unwrap_or(0)
            + counts
                .get(&OfficialCommitteePackSourceKind::SyntheticTest)
                .copied()
                .unwrap_or(0);
        let row_level_count = rows
            .iter()
            .filter(|row| {
                row.materialization_level == CommitteeScenarioMaterializationLevel::RowLevel
            })
            .count();
        let summary_derived_count = rows.iter().filter(|row| is_summary_derived(row)).count();
        let outcome_linked_count = rows
            .iter()
            .filter(|row| row.outcome_reference.is_some())
            .count();
        let baseline_reference_count = rows
            .iter()
            .filter(|row| row.baseline_signal_summary.is_some())
            .count();
        let external_reference_count = rows
            .iter()
            .filter(|row| row.external_prediction_summary.is_some())
            .count();
        let no_trade_counterfactual_count = rows
            .iter()
            .filter(|row| row.no_trade_counterfactual.is_some())
            .count();
        let risk_denial_counterfactual_count = rows
            .iter()
            .filter(|row| row.risk_denial_counterfactual.is_some())
            .count();
        Ok(OfficialCommitteeScenarioPack {
            pack_id: config.pack_id.clone(),
            rows,
            source_summary: build_source_summary(&counts),
            official_row_count,
            crypto_only_row_count,
            yfinance_row_count,
            fixture_row_count,
            row_level_count,
            summary_derived_count,
            outcome_linked_count,
            baseline_reference_count,
            external_reference_count,
            no_trade_counterfactual_count,
            risk_denial_counterfactual_count,
            storage_bytes,
            reason_codes: config
                .reason_codes
                .iter()
                .cloned()
                .chain([
                    ReasonCode::OfficialCommitteePackBuilt,
                    ReasonCode::OfficialCommitteePackFiltered,
                ])
                .collect(),
        })
    }
}

impl OfficialCommitteeScenarioPack {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn summary_derived_ratio(&self) -> f64 {
        self.summary_derived_count as f64 / self.row_count().max(1) as f64
    }

    pub fn research_only_ratio(&self) -> f64 {
        self.yfinance_row_count as f64 / self.row_count().max(1) as f64
    }

    pub fn fixture_ratio(&self) -> f64 {
        self.fixture_row_count as f64 / self.row_count().max(1) as f64
    }

    pub fn crypto_only_ratio(&self) -> f64 {
        self.crypto_only_row_count as f64 / self.row_count().max(1) as f64
    }

    pub fn row_level_ratio(&self) -> f64 {
        self.row_level_count as f64 / self.row_count().max(1) as f64
    }

    pub fn to_committee_scenario_set(&self) -> CommitteeScenarioSet {
        CommitteeScenarioSet {
            scenario_id: self.pack_id.clone(),
            rows: self.rows.clone(),
            source_summary: self.source_summary.clone(),
            row_count: self.rows.len(),
            official_row_count: self.official_row_count,
            research_only_row_count: self.yfinance_row_count,
            fixture_row_count: self.fixture_row_count,
            skipped_row_count: 0,
            reason_codes: self.reason_codes.clone(),
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("pack_id={}", self.pack_id),
            format!("source_summary={}", self.source_summary),
            format!("row_count={}", self.row_count()),
            format!("official_row_count={}", self.official_row_count),
            format!("crypto_only_row_count={}", self.crypto_only_row_count),
            format!("yfinance_row_count={}", self.yfinance_row_count),
            format!("fixture_row_count={}", self.fixture_row_count),
            format!("row_level_count={}", self.row_level_count),
            format!("summary_derived_count={}", self.summary_derived_count),
            format!("outcome_linked_count={}", self.outcome_linked_count),
            format!("baseline_reference_count={}", self.baseline_reference_count),
            format!("external_reference_count={}", self.external_reference_count),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denial_counterfactual_count={}",
                self.risk_denial_counterfactual_count
            ),
            format!("storage_bytes={}", self.storage_bytes),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_scenario_pack.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_scenario_pack.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn row_allowed(
    row: &CommitteeScenarioRow,
    config: &OfficialCommitteeScenarioPackConfig,
    has_any_preflight: bool,
) -> bool {
    let class = classify_row(row);
    if !config.allowed_source_kinds.contains(&class) {
        return false;
    }
    if matches!(class, OfficialCommitteePackSourceKind::YFinanceResearch)
        && !config.allow_yfinance_research
    {
        return false;
    }
    if matches!(
        class,
        OfficialCommitteePackSourceKind::Fixture | OfficialCommitteePackSourceKind::SyntheticTest
    ) && !config.allow_fixture
    {
        return false;
    }
    if row.market == ProviderMarket::Crypto && !config.allow_crypto_only {
        return false;
    }
    if !config.allow_summary_derived_rows && is_summary_derived(row) {
        return false;
    }
    if config.require_outcome_reference && row.outcome_reference.is_none() {
        return false;
    }
    let requires_strict_official_checks =
        matches!(class, OfficialCommitteePackSourceKind::OfficialApiCollected)
            && row.market != ProviderMarket::Crypto;
    if config.require_provenance
        && requires_strict_official_checks
        && row
            .provenance_summary
            .to_ascii_lowercase()
            .contains("missing")
    {
        return false;
    }
    if config.require_preflight && requires_strict_official_checks && !has_any_preflight {
        return false;
    }
    true
}

pub fn classify_row(row: &CommitteeScenarioRow) -> OfficialCommitteePackSourceKind {
    if matches!(
        row.source_kind,
        CommitteeScenarioSourceKind::Fixture | CommitteeScenarioSourceKind::SyntheticTest
    ) {
        return if row.source_kind == CommitteeScenarioSourceKind::SyntheticTest {
            OfficialCommitteePackSourceKind::SyntheticTest
        } else {
            OfficialCommitteePackSourceKind::Fixture
        };
    }
    match row.evidence_source_kind {
        EvidenceSourceKind::YFinanceResearch => OfficialCommitteePackSourceKind::YFinanceResearch,
        EvidenceSourceKind::OfficialApiCollected if row.market == ProviderMarket::Crypto => {
            OfficialCommitteePackSourceKind::OfficialCryptoPublic
        }
        EvidenceSourceKind::OfficialApiCollected | EvidenceSourceKind::RealLocal => {
            OfficialCommitteePackSourceKind::OfficialApiCollected
        }
        _ => OfficialCommitteePackSourceKind::Unknown,
    }
}

fn counts_for_rows(
    rows: &[CommitteeScenarioRow],
) -> BTreeMap<OfficialCommitteePackSourceKind, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(classify_row(row)).or_insert(0) += 1;
    }
    counts
}

fn build_source_summary(counts: &BTreeMap<OfficialCommitteePackSourceKind, usize>) -> String {
    counts
        .iter()
        .map(|(kind, count)| format!("{:?}={count}", kind))
        .collect::<Vec<_>>()
        .join("|")
}

fn is_summary_derived(row: &CommitteeScenarioRow) -> bool {
    row.reason_codes.contains(&ReasonCode::SummaryDerived)
        || row.materialization_level != CommitteeScenarioMaterializationLevel::RowLevel
}

fn default_allowed_source_kinds() -> Vec<OfficialCommitteePackSourceKind> {
    vec![
        OfficialCommitteePackSourceKind::OfficialApiCollected,
        OfficialCommitteePackSourceKind::OfficialCryptoPublic,
    ]
}

fn default_true() -> bool {
    true
}

fn default_max_rows() -> usize {
    100
}

fn default_max_symbols() -> usize {
    50
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn is_remote_path(path: &str) -> bool {
    path.contains("://")
}
