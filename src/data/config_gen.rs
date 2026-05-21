use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;
use crate::experiment::{
    AblationDimension, AblationOverride, AblationStudyConfig, AblationValue, AblationVariant,
    ExperimentMatrixConfig, ExperimentMode, ExperimentVariant, ExperimentVariantOverrides,
    RealEvidenceClosureConfig,
};
use crate::feature::FeatureConfig;
use crate::{
    ChairConfig, DataValidationConfig, DatasetBundleConfig, DatasetEntry, GovernorConfig,
    NoTradeScoreConfig, RegimeClassifierConfig,
};

use super::EvidenceSourceKind;
use super::{CandleCsvFormat, LocalDataOnboardingConfig, PreflightFinalStatus, PreflightReport};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigGenerationPolicy {
    #[default]
    ReadyOnly,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratedConfigBundle {
    pub dataset_entry_toml: String,
    pub real_evidence_closure_toml: String,
    #[serde(default)]
    pub batch_matrix_toml: Option<String>,
    #[serde(default)]
    pub ablation_study_toml: Option<String>,
    pub notes: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl GeneratedConfigBundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("generated_real_local_dataset_entry.toml"),
            &self.dataset_entry_toml,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("generated_real_evidence_closure.toml"),
            &self.real_evidence_closure_toml,
        )
        .map_err(|err| err.to_string())?;
        if let Some(batch) = &self.batch_matrix_toml {
            fs::write(output_dir.join("generated_batch_matrix.toml"), batch)
                .map_err(|err| err.to_string())?;
        }
        if let Some(ablation) = &self.ablation_study_toml {
            fs::write(output_dir.join("generated_ablation_study.toml"), ablation)
                .map_err(|err| err.to_string())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealEvidenceRerunPlan {
    pub preflight_report: PreflightReport,
    #[serde(default)]
    pub generated_config_bundle: Option<GeneratedConfigBundle>,
    pub suggested_commands: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub expected_minimum_targets: Vec<String>,
    pub caveats: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl RealEvidenceRerunPlan {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("preflight_status={:?}", self.preflight_report.final_status),
            format!("suggested_commands={}", self.suggested_commands.join(" | ")),
            format!("expected_outputs={}", self.expected_outputs.join(" | ")),
            format!(
                "expected_minimum_targets={}",
                self.expected_minimum_targets.join(" | ")
            ),
            format!("caveats={}", self.caveats.join(" | ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.preflight_report.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("real_evidence_rerun_plan.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("real_evidence_rerun_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        if let Some(bundle) = &self.generated_config_bundle {
            bundle.write_to_dir(output_dir)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct EntryListFile {
    entries: Vec<DatasetEntry>,
}

pub fn generate_config_bundle(
    config: &LocalDataOnboardingConfig,
    report: &PreflightReport,
    policy: ConfigGenerationPolicy,
) -> Option<GeneratedConfigBundle> {
    let ready = report.final_status == PreflightFinalStatus::ReadyForRealEvidence;
    if !ready && policy == ConfigGenerationPolicy::ReadyOnly {
        return None;
    }
    let dataset_entry = build_dataset_entry(config, report);
    let dataset_entry_toml = toml::to_string_pretty(&EntryListFile {
        entries: vec![dataset_entry.clone()],
    })
    .unwrap_or_else(|_| render_dataset_entry_file(&dataset_entry));
    if !ready {
        return Some(GeneratedConfigBundle {
            dataset_entry_toml,
            real_evidence_closure_toml:
                "# diagnostic only: preflight is not ready for real-evidence execution\n"
                    .to_string(),
            batch_matrix_toml: None,
            ablation_study_toml: None,
            notes: vec![
                "diagnostic only: preflight must reach ReadyForRealEvidence first".to_string(),
            ],
            reason_codes: vec![ReasonCode::GeneratedConfigBundleBuilt],
        });
    }

    let real_evidence = build_real_evidence_config(config);
    let batch_matrix = build_batch_matrix(config, &dataset_entry);
    let ablation = build_ablation_study(config);
    Some(GeneratedConfigBundle {
        dataset_entry_toml,
        real_evidence_closure_toml: toml::to_string_pretty(&real_evidence)
            .unwrap_or_else(|_| render_real_evidence_config(&real_evidence)),
        batch_matrix_toml: Some(
            toml::to_string_pretty(&batch_matrix)
                .unwrap_or_else(|err| format!("# failed to serialize batch matrix: {err}\n")),
        ),
        ablation_study_toml: Some(
            toml::to_string_pretty(&ablation)
                .unwrap_or_else(|err| format!("# failed to serialize ablation study: {err}\n")),
        ),
        notes: vec![
            "local-only generated config bundle".to_string(),
            if config.source_kind == Some(EvidenceSourceKind::OfficialApiCollected) {
                "official-api-collected provenance passed local preflight eligibility".to_string()
            } else {
                "user_supplied real-local provenance is required".to_string()
            },
            "no live API, broker, downloader, or LLM fields are present".to_string(),
        ],
        reason_codes: vec![ReasonCode::GeneratedConfigBundleBuilt],
    })
}

pub fn build_real_evidence_rerun_plan(
    config: &LocalDataOnboardingConfig,
    report: PreflightReport,
    policy: ConfigGenerationPolicy,
) -> RealEvidenceRerunPlan {
    let bundle = generate_config_bundle(config, &report, policy);
    let mut suggested_commands = Vec::new();
    let mut expected_outputs = vec![
        "preflight_report.json".to_string(),
        "preflight_report.txt".to_string(),
        "real_evidence_rerun_plan.json".to_string(),
        "real_evidence_rerun_plan.txt".to_string(),
    ];
    if bundle.is_some() {
        expected_outputs.push("generated_real_local_dataset_entry.toml".to_string());
        expected_outputs.push("generated_real_evidence_closure.toml".to_string());
    }
    if bundle
        .as_ref()
        .and_then(|value| value.batch_matrix_toml.as_ref())
        .is_some()
    {
        expected_outputs.push("generated_batch_matrix.toml".to_string());
    }
    if bundle
        .as_ref()
        .and_then(|value| value.ablation_study_toml.as_ref())
        .is_some()
    {
        expected_outputs.push("generated_ablation_study.toml".to_string());
    }
    if report.final_status == PreflightFinalStatus::ReadyForRealEvidence {
        suggested_commands.push(
            "soma-experiment real-evidence --config <out>/generated_real_evidence_closure.toml"
                .to_string(),
        );
        suggested_commands
            .push("soma-experiment batch --config <out>/generated_batch_matrix.toml".to_string());
        suggested_commands.push(
            "soma-experiment ablation --config <out>/generated_ablation_study.toml".to_string(),
        );
    }
    RealEvidenceRerunPlan {
        preflight_report: report,
        generated_config_bundle: bundle,
        suggested_commands,
        expected_outputs,
        expected_minimum_targets: vec![
            format!("usable datasets >= {}", config.target_min_usable_datasets),
            format!("outcome records >= {}", config.target_min_outcomes),
            format!(
                "comparable variants >= {}",
                config.target_min_comparable_variants
            ),
        ],
        caveats: vec![
            "research-only, local-only workflow".to_string(),
            "synthetic/test evidence still does not count as real-market readiness".to_string(),
            "no live trading, no real-money recommendation".to_string(),
            "user must supply and license local market CSV data".to_string(),
        ],
        reason_codes: vec![ReasonCode::RealEvidenceRerunPlanned],
    }
}

fn build_dataset_entry(
    config: &LocalDataOnboardingConfig,
    report: &PreflightReport,
) -> DatasetEntry {
    let dataset_id = report
        .data_manifest_preview
        .as_ref()
        .map(|manifest| manifest.dataset_id.clone())
        .unwrap_or_else(|| {
            format!(
                "real_local_{:08x}",
                crate::core::stable_hash(&config.input_path)
            )
        });
    let csv_format = report.detected_format.clone().unwrap_or(
        config
            .csv_format_hint
            .clone()
            .unwrap_or(CandleCsvFormat::GenericOhlcv),
    );
    let source_kind = config.source_kind.unwrap_or(EvidenceSourceKind::RealLocal);
    let mut tags = vec!["real-local".to_string()];
    if source_kind == EvidenceSourceKind::OfficialApiCollected {
        tags = vec![
            "official-api-collected".to_string(),
            "auto-collected".to_string(),
        ];
    } else if config.user_supplied {
        tags.push("user-supplied".to_string());
    }
    DatasetEntry {
        dataset_id,
        symbol: report.symbol.clone(),
        data_path: config.input_path.clone(),
        csv_format: csv_format.clone(),
        timeframe: report.timeframe,
        resample_to: None,
        venue: config.resolved_venue(Some(&csv_format)),
        asset_class: config.resolved_asset_class(),
        provenance: Some(report.provenance.clone()),
        enabled: true,
        tags,
        expected_min_rows: Some(config.min_rows_for_preflight),
        notes: Some("Generated by Sprint 17 onboarding.".to_string()),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn build_real_evidence_config(config: &LocalDataOnboardingConfig) -> RealEvidenceClosureConfig {
    RealEvidenceClosureConfig {
        closure_id: format!("{}-real-evidence", config.onboarding_id),
        dataset_bundle_config_path: Some("generated_real_local_dataset_entry.toml".to_string()),
        real_dataset_entries: Vec::new(),
        source_sprint15_report_path: Some(
            "target/soma_evidence_closure/sprint15_evidence_closure/evidence_closure_report.json"
                .to_string(),
        ),
        output_root: format!("{}/generated_real_evidence", config.output_root),
        evidence_store_path: format!("{}/generated_real_evidence_store", config.output_root),
        min_real_local_datasets: config.target_min_usable_datasets,
        min_real_local_outcome_records: config.target_min_outcomes,
        min_real_local_comparable_variants: config.target_min_comparable_variants,
        allow_synthetic_for_pipeline_smoke: true,
        allow_synthetic_for_readiness: false,
        continue_on_failure: true,
        strict_data_quality: config.strict,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn build_batch_matrix(
    config: &LocalDataOnboardingConfig,
    dataset_entry: &DatasetEntry,
) -> ExperimentMatrixConfig {
    ExperimentMatrixConfig {
        matrix_id: format!("{}-real-local-batch", config.onboarding_id),
        dataset_bundle: DatasetBundleConfig {
            bundle_id: format!("{}-bundle", config.onboarding_id),
            entries: vec![dataset_entry.clone()],
            default_data_validation_config: DataValidationConfig {
                strict: config.strict,
                allow_sort_repair: config.allow_sort_repair,
                allow_duplicate_drop: config.allow_duplicate_drop,
                allow_gap: true,
                max_gap_count: usize::MAX,
                max_gap_ratio: 1.0,
                max_invalid_ratio: 1.0,
                expected_step_ms: Some(match config.resolved_timeframe() {
                    Timeframe::OneMinute => 60_000,
                    Timeframe::FiveMinute => 300_000,
                    Timeframe::FifteenMinute => 900_000,
                    Timeframe::OneHour => 3_600_000,
                    Timeframe::OneDay => 86_400_000,
                    Timeframe::Custom { seconds } => u64::from(seconds) * 1_000,
                }),
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            default_feature_config: FeatureConfig::default(),
            default_regime_config: RegimeClassifierConfig::default(),
            default_chair_config: ChairConfig::default(),
            default_walk_forward_config: config.resolved_walk_forward_config(),
            default_triple_barrier_config: config.resolved_triple_barrier_config(),
            default_cost_model: config.resolved_cost_model(),
            default_no_trade_score_config: NoTradeScoreConfig::default(),
            default_risk_config: GovernorConfig::default(),
            output_root: format!("{}/generated_batch_runs", config.output_root),
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        variants: vec![ExperimentVariant {
            variant_id: "baseline_1m".to_string(),
            mode: ExperimentMode::BaselineOnly,
            overrides: ExperimentVariantOverrides {
                timeframe: Some(config.resolved_timeframe()),
                ..ExperimentVariantOverrides::default()
            },
            enabled: true,
            tags: vec!["real-local".to_string(), "baseline".to_string()],
            reason_codes: vec![ReasonCode::DeterministicPath],
        }],
        continue_on_failure: true,
        require_all_pass: false,
        deterministic_run_id: Some(format!("{}-seed", config.onboarding_id)),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn build_ablation_study(config: &LocalDataOnboardingConfig) -> AblationStudyConfig {
    AblationStudyConfig {
        study_id: format!("{}-ablation", config.onboarding_id),
        description: Some("Generated by Sprint 17 onboarding rerun plan.".to_string()),
        base_matrix_config_path: Some("generated_batch_matrix.toml".to_string()),
        embedded_base_matrix: None,
        output_root: format!("{}/generated_ablation", config.output_root),
        require_baseline_pass: true,
        continue_on_variant_failure: true,
        variants: vec![
            AblationVariant {
                variant_id: "volume_off".to_string(),
                dimension: AblationDimension::FeatureGroup,
                overrides: vec![AblationOverride {
                    target: "volume".to_string(),
                    value: AblationValue::Bool(false),
                }],
                research_only: false,
                enabled: true,
                tags: vec!["closure".to_string()],
                notes: None,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            AblationVariant {
                variant_id: "higher_cost".to_string(),
                dimension: AblationDimension::CostModel,
                overrides: vec![AblationOverride {
                    target: "spread_bps".to_string(),
                    value: AblationValue::Float(4.0),
                }],
                research_only: false,
                enabled: true,
                tags: vec!["closure".to_string()],
                notes: None,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
        ],
        created_at_ms: Some(42),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn render_dataset_entry_file(entry: &DatasetEntry) -> String {
    let csv_format = format!("{:?}", entry.csv_format);
    let timeframe = format!("{:?}", entry.timeframe);
    let venue = format!("{:?}", entry.venue);
    let asset_class = format!("{:?}", entry.asset_class);
    let expected_min_rows = entry.expected_min_rows.unwrap_or(0);
    let provenance = entry.provenance.as_ref();
    [
        "[[entries]]".to_string(),
        format!("dataset_id = {:?}", entry.dataset_id),
        format!("symbol = {:?}", entry.symbol),
        format!("data_path = {:?}", entry.data_path),
        format!("csv_format = {:?}", csv_format),
        format!("timeframe = {:?}", timeframe),
        format!("venue = {:?}", venue),
        format!("asset_class = {:?}", asset_class),
        format!("enabled = {}", entry.enabled),
        format!("tags = {:?}", entry.tags),
        format!("expected_min_rows = {expected_min_rows}"),
        "reason_codes = [\"DeterministicPath\"]".to_string(),
        "".to_string(),
        "[entries.provenance]".to_string(),
        format!(
            "source_kind = {:?}",
            provenance
                .map(|value| format!("{:?}", value.source_kind))
                .unwrap_or_else(|| "RealLocal".to_string())
        ),
        format!(
            "source_label = {:?}",
            provenance
                .map(|value| value.source_label.clone())
                .unwrap_or_else(|| "user-local-generated".to_string())
        ),
        format!(
            "local_path = {:?}",
            provenance
                .and_then(|value| value.local_path.clone())
                .unwrap_or_else(|| entry.data_path.clone())
        ),
        format!(
            "user_supplied = {}",
            provenance.map(|value| value.user_supplied).unwrap_or(true)
        ),
        "downloaded_by_soma = false".to_string(),
        "remote_url_present = false".to_string(),
        "reason_codes = [\"DeterministicPath\"]".to_string(),
    ]
    .join("\n")
}

fn render_real_evidence_config(config: &RealEvidenceClosureConfig) -> String {
    [
        format!("closure_id = {:?}", config.closure_id),
        format!(
            "dataset_bundle_config_path = {:?}",
            config.dataset_bundle_config_path
        ),
        format!(
            "source_sprint15_report_path = {:?}",
            config.source_sprint15_report_path
        ),
        format!("output_root = {:?}", config.output_root),
        format!("evidence_store_path = {:?}", config.evidence_store_path),
        format!(
            "min_real_local_datasets = {}",
            config.min_real_local_datasets
        ),
        format!(
            "min_real_local_outcome_records = {}",
            config.min_real_local_outcome_records
        ),
        format!(
            "min_real_local_comparable_variants = {}",
            config.min_real_local_comparable_variants
        ),
        format!(
            "allow_synthetic_for_pipeline_smoke = {}",
            config.allow_synthetic_for_pipeline_smoke
        ),
        format!(
            "allow_synthetic_for_readiness = {}",
            config.allow_synthetic_for_readiness
        ),
        format!("continue_on_failure = {}", config.continue_on_failure),
        format!("strict_data_quality = {}", config.strict_data_quality),
        "reason_codes = [\"DeterministicPath\"]".to_string(),
    ]
    .join("\n")
}
