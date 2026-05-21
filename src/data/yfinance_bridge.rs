use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;

use super::{
    AssetClass, CandleCsvFormat, ConfigGenerationPolicy, DataProvenance, EvidenceSourceKind,
    GeneratedConfigBundle, LocalDataOnboardingConfig, MarketVenue, PreflightFinalStatus,
    PreflightReport, PreflightValidator, build_real_evidence_rerun_plan,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YFinanceImportConfig {
    pub import_id: String,
    pub canonical_csv_path: String,
    pub output_root: String,
    pub symbol: String,
    pub venue: MarketVenue,
    pub asset_class: AssetClass,
    pub timeframe: Timeframe,
    #[serde(default)]
    pub provenance_path: Option<String>,
    #[serde(default)]
    pub manifest_path: Option<String>,
    #[serde(default)]
    pub source_label: Option<String>,
    #[serde(default = "default_diagnostic_only")]
    pub config_generation_policy: ConfigGenerationPolicy,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct YFinanceResearchManifest {
    pub manifest_version: u32,
    pub source_kind: EvidenceSourceKind,
    pub provider_label: String,
    pub upstream_label: String,
    pub symbol: String,
    pub interval: String,
    pub row_count: usize,
    pub first_timestamp_ms: u64,
    pub last_timestamp_ms: u64,
    pub adjusted_price_policy: String,
    pub corporate_action_adjusted: bool,
    pub canonical_csv: String,
    pub provenance_path: String,
    pub readiness_eligible: bool,
    pub benchmark_eligible: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct YFinancePreflightBridge {
    pub source_kind: EvidenceSourceKind,
    pub provenance: DataProvenance,
    pub local_onboarding_config: LocalDataOnboardingConfig,
    pub preflight_report: PreflightReport,
    pub benchmark_eligible: bool,
    pub official_readiness_eligible: bool,
    pub generated_config_paths: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for YFinanceImportConfig {
    fn default() -> Self {
        Self {
            import_id: "yfinance_import".to_string(),
            canonical_csv_path: "research/output/aapl_1d/canonical/aapl_1d.csv".to_string(),
            output_root: "target/soma_yfinance_import".to_string(),
            symbol: "AAPL".to_string(),
            venue: MarketVenue::NASDAQ,
            asset_class: AssetClass::Equity,
            timeframe: Timeframe::OneDay,
            provenance_path: Some(
                "research/output/aapl_1d/provenance/aapl_1d.provenance.json".to_string(),
            ),
            manifest_path: Some(
                "research/output/aapl_1d/manifests/aapl_1d.manifest.json".to_string(),
            ),
            source_label: Some("yfinance-aapl-1d".to_string()),
            config_generation_policy: ConfigGenerationPolicy::DiagnosticOnly,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl YFinanceImportConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&contents)
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.import_id)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            Some(self.canonical_csv_path.as_str()),
            Some(self.output_root.as_str()),
            self.provenance_path.as_deref(),
            self.manifest_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                reasons.push(ReasonCode::LocalPathRejected);
            }
        }
        dedupe_reasons(reasons)
    }
}

impl YFinancePreflightBridge {
    pub fn to_text(&self) -> String {
        [
            format!("source_kind={:?}", self.source_kind),
            format!("benchmark_eligible={}", self.benchmark_eligible),
            format!(
                "official_readiness_eligible={}",
                self.official_readiness_eligible
            ),
            format!(
                "generated_config_paths={}",
                self.generated_config_paths.join(" | ")
            ),
            format!("warnings={}", self.warnings.join(" | ")),
            self.preflight_report.to_text(),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("yfinance_preflight_bridge.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("yfinance_preflight_bridge.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

pub fn build_yfinance_local_onboarding_config(
    config: &YFinanceImportConfig,
) -> LocalDataOnboardingConfig {
    LocalDataOnboardingConfig {
        onboarding_id: format!("yfinance-{}", config.import_id),
        input_path: config.canonical_csv_path.clone(),
        output_root: config.output_dir().display().to_string(),
        symbol: Some(config.symbol.clone()),
        venue: Some(config.venue),
        asset_class: Some(config.asset_class),
        timeframe: Some(config.timeframe),
        csv_format_hint: Some(CandleCsvFormat::GenericOhlcv),
        custom_column_map: None,
        source_kind: Some(EvidenceSourceKind::YFinanceResearch),
        user_supplied: true,
        source_label: config
            .source_label
            .clone()
            .or_else(|| Some(format!("yfinance-{}", config.import_id))),
        strict: true,
        allow_format_autodetect: false,
        allow_sort_repair: false,
        allow_duplicate_drop: false,
        min_rows_for_preflight: 40,
        target_min_outcomes: 20,
        target_min_comparable_variants: 2,
        target_min_usable_datasets: 1,
        walk_forward_config: None,
        triple_barrier_config: None,
        cost_model: None,
        reason_codes: vec![
            ReasonCode::DeterministicPath,
            ReasonCode::YFinanceUnofficialEvidence,
        ],
    }
}

pub fn run_yfinance_preflight_bridge(
    config: &YFinanceImportConfig,
) -> Result<YFinancePreflightBridge, String> {
    if !config.validate_local_paths().is_empty() {
        return Err("yfinance-import paths must be local".to_string());
    }
    let output_dir = config.output_dir();
    fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

    let onboarding = build_yfinance_local_onboarding_config(config);
    let mut provenance = load_yfinance_provenance(config, &onboarding)?;
    let manifest = load_yfinance_manifest(config)?;
    let mut preflight_report = PreflightValidator::default().run(&onboarding);

    let benchmark_eligible = manifest
        .as_ref()
        .map(|value| value.benchmark_eligible)
        .unwrap_or_else(|| preflight_supports_research_benchmark(&preflight_report));
    provenance.benchmark_eligible = Some(benchmark_eligible);
    provenance.readiness_eligible = Some(false);
    provenance.notes = Some(
        "Sprint 27 research-only yfinance bridge; do not count toward official readiness."
            .to_string(),
    );
    preflight_report.provenance = provenance.clone();
    if let Some(preview) = preflight_report.data_manifest_preview.as_mut() {
        preview.source_kind = EvidenceSourceKind::YFinanceResearch;
        preview.provenance = Some(provenance.clone());
        preview.auth_requirement_summary = Some(
            "No API key required for fixture mode; unofficial research-only source.".to_string(),
        );
        preview
            .reason_codes
            .push(ReasonCode::YFinanceUnofficialEvidence);
    }

    let mut generated_config_paths = Vec::new();
    if benchmark_eligible {
        let plan = build_real_evidence_rerun_plan(
            &onboarding,
            preflight_report.clone(),
            config.config_generation_policy,
        );
        let generated_dir = output_dir.join("generated_config");
        plan.write_to_dir(&generated_dir)?;
        generated_config_paths =
            gather_generated_config_paths(&generated_dir, &plan.generated_config_bundle);
    }

    let bridge = YFinancePreflightBridge {
        source_kind: EvidenceSourceKind::YFinanceResearch,
        provenance,
        local_onboarding_config: onboarding,
        preflight_report,
        benchmark_eligible,
        official_readiness_eligible: false,
        generated_config_paths,
        warnings: vec![
            "yfinance data is research-only and unofficial in this repository".to_string(),
            "yfinance evidence does not count as official API coverage or readiness evidence"
                .to_string(),
        ],
        reason_codes: vec![
            ReasonCode::YFinanceBridgeBuilt,
            ReasonCode::YFinanceUnofficialEvidence,
        ],
    };
    bridge.write_to_dir(&output_dir)?;
    Ok(bridge)
}

fn load_yfinance_provenance(
    config: &YFinanceImportConfig,
    onboarding: &LocalDataOnboardingConfig,
) -> Result<DataProvenance, String> {
    if let Some(path) = config.provenance_path.as_deref() {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let mut provenance: DataProvenance =
            serde_json::from_str(&text).map_err(|err| err.to_string())?;
        provenance.source_kind = EvidenceSourceKind::YFinanceResearch;
        provenance.source_label = config
            .source_label
            .clone()
            .unwrap_or_else(|| provenance.source_label.clone());
        provenance.local_path = Some(config.canonical_csv_path.clone());
        provenance.remote_url_present = false;
        provenance.downloaded_by_soma = false;
        provenance.provider_label = Some("yfinance".to_string());
        provenance.upstream_label = Some("Yahoo Finance".to_string());
        provenance.official_provider = Some(false);
        provenance.affiliated_or_endorsed = Some(false);
        provenance.intended_use =
            Some("research-only unofficial supplemental benchmark data".to_string());
        provenance.readiness_eligible = Some(false);
        return Ok(provenance);
    }

    let mut provenance = onboarding.build_provenance();
    provenance.provider_label = Some("yfinance".to_string());
    provenance.upstream_label = Some("Yahoo Finance".to_string());
    provenance.official_provider = Some(false);
    provenance.affiliated_or_endorsed = Some(false);
    provenance.intended_use =
        Some("research-only unofficial supplemental benchmark data".to_string());
    provenance.readiness_eligible = Some(false);
    provenance.benchmark_eligible = Some(false);
    provenance
        .reason_codes
        .push(ReasonCode::YFinanceUnofficialEvidence);
    Ok(provenance)
}

fn load_yfinance_manifest(
    config: &YFinanceImportConfig,
) -> Result<Option<YFinanceResearchManifest>, String> {
    let Some(path) = config.manifest_path.as_deref() else {
        return Ok(None);
    };
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let manifest = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    Ok(Some(manifest))
}

fn preflight_supports_research_benchmark(report: &PreflightReport) -> bool {
    !matches!(
        report.final_status,
        PreflightFinalStatus::MissingFile
            | PreflightFinalStatus::UnsupportedFormat
            | PreflightFinalStatus::AmbiguousFormat
            | PreflightFinalStatus::DataQualityTooLow
    )
}

fn gather_generated_config_paths(
    output_dir: &Path,
    bundle: &Option<GeneratedConfigBundle>,
) -> Vec<String> {
    let mut paths = vec![
        output_dir
            .join("real_evidence_rerun_plan.json")
            .display()
            .to_string(),
        output_dir
            .join("real_evidence_rerun_plan.txt")
            .display()
            .to_string(),
        output_dir
            .join("preflight_report.json")
            .display()
            .to_string(),
        output_dir
            .join("preflight_report.txt")
            .display()
            .to_string(),
    ];
    if bundle.is_some() {
        paths.push(
            output_dir
                .join("generated_real_local_dataset_entry.toml")
                .display()
                .to_string(),
        );
        paths.push(
            output_dir
                .join("generated_real_evidence_closure.toml")
                .display()
                .to_string(),
        );
    }
    if bundle
        .as_ref()
        .and_then(|value| value.batch_matrix_toml.as_ref())
        .is_some()
    {
        paths.push(
            output_dir
                .join("generated_batch_matrix.toml")
                .display()
                .to_string(),
        );
    }
    if bundle
        .as_ref()
        .and_then(|value| value.ablation_study_toml.as_ref())
        .is_some()
    {
        paths.push(
            output_dir
                .join("generated_ablation_study.toml")
                .display()
                .to_string(),
        );
    }
    paths
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}

fn default_diagnostic_only() -> ConfigGenerationPolicy {
    ConfigGenerationPolicy::DiagnosticOnly
}
