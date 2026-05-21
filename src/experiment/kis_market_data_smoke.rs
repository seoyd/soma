use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::security::{SecretRedactionAuditConfig, SecretRedactionAuditRunner};
use crate::ui::{ControlTowerAutoRefreshConfig, ControlTowerAutoRefreshRunner};

use super::environment_isolation::{EnvironmentIsolationConfig, EnvironmentIsolationRunner};
use super::kis_auth_closure::{
    KISAuthClosureConfig, KISAuthClosureReport, KISAuthClosureRunner, KISAuthClosureStatus,
};
use super::kis_collection_plan_v2::{
    KISCollectionPlanV2, KISCollectionPlanV2Config, KISCollectionPlanV2Runner,
    KISCollectionPlanV2Status,
};
use super::kis_evidence_depth::{KISEvidenceDepthRunConfig, KISEvidenceDepthRunRunner};
use super::kis_market_data_activation::{
    KISMarketDataActivationConfig, KISOfficialMarketDataActivationRunner,
};
use super::kis_market_data_dry_run::{
    KISMarketDataDryRunConfig, KISMarketDataDryRunReport, KISMarketDataDryRunRunner,
    KISMarketDataDryRunStatus,
};
use super::kis_market_data_smoke_bundle::KISMarketDataSmokeControlTowerBundle;
use super::operational_runbook_v2::{OperationalRunbookV2Config, OperationalRunbookV2Runner};

fn default_output_root() -> String {
    "target/soma_kis_market_data_smoke".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_bytes() -> usize {
    10_000_000
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISMarketDataEvidenceSmokeConfig {
    pub smoke_id: String,
    #[serde(default)]
    pub auth_closure_config_path: Option<String>,
    #[serde(default)]
    pub dry_run_config_path: Option<String>,
    #[serde(default)]
    pub collection_plan_v2_config_path: Option<String>,
    #[serde(default)]
    pub endpoint_policy_path: Option<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_auth_closure: bool,
    #[serde(default = "default_true")]
    pub run_dry_run: bool,
    #[serde(default = "default_true")]
    pub run_collection_plan: bool,
    #[serde(default = "default_true")]
    pub run_fixture_replay: bool,
    #[serde(default = "default_true")]
    pub run_local_import: bool,
    #[serde(default)]
    pub run_operator_live_collection: bool,
    #[serde(default = "default_true")]
    pub run_schema_drift: bool,
    #[serde(default = "default_true")]
    pub run_canonical_validation: bool,
    #[serde(default = "default_true")]
    pub run_preflight: bool,
    #[serde(default = "default_true")]
    pub run_candle_sufficiency: bool,
    #[serde(default = "default_true")]
    pub run_outcome_link_closure: bool,
    #[serde(default = "default_true")]
    pub run_complete_row_closure: bool,
    #[serde(default = "default_true")]
    pub run_evidence_depth: bool,
    #[serde(default = "default_true")]
    pub run_trinity_loop: bool,
    #[serde(default = "default_true")]
    pub run_control_tower_refresh: bool,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISMarketDataEvidenceSmokeFinalStatus {
    KISAuthClosed,
    KISDryRunReady,
    KISBoundedSmokeReady,
    KISMarketDataEvidenceImproved,
    KISOutcomeLinksImproved,
    KISCounterfactualsImproved,
    KISCompleteRowsImproved,
    KISAuthMissing,
    KISBaseUrlMissing,
    EndpointPolicyBlocked,
    StillNeedKISMarketData,
    StillNeedOutcomeLinkDepth,
    StillNeedCounterfactualDepth,
    NoImprovement,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISMarketDataEvidenceSmokeRecommendation {
    SetKISAppKey,
    SetKISAppSecret,
    SetKISBaseUrl,
    RunKISDryRun,
    RunKISCollectionPlan,
    RunKISMarketDataActivate,
    RunKISEvidenceDepth,
    RunControlTowerRefresh,
    RunTrinityOperationalLoop,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISMarketDataEvidenceSmokeReport {
    pub smoke_id: String,
    pub auth_closure_status: KISAuthClosureStatus,
    pub dry_run_status: KISMarketDataDryRunStatus,
    pub collection_plan_status: KISCollectionPlanV2Status,
    #[serde(default)]
    pub schema_drift_status: Option<String>,
    #[serde(default)]
    pub canonical_validation_status: Option<String>,
    #[serde(default)]
    pub preflight_status: Option<String>,
    #[serde(default)]
    pub candle_sufficiency_status: Option<String>,
    #[serde(default)]
    pub outcome_link_closure_status: Option<String>,
    #[serde(default)]
    pub complete_row_closure_status: Option<String>,
    #[serde(default)]
    pub evidence_depth_status: Option<String>,
    #[serde(default)]
    pub trinity_loop_status: Option<String>,
    #[serde(default)]
    pub control_tower_refresh_status: Option<String>,
    pub added_canonical_csvs: usize,
    pub added_official_rows: usize,
    pub added_preflight_ready_rows: usize,
    pub added_official_ready_candles: usize,
    pub added_outcome_links: usize,
    pub added_counterfactuals: usize,
    pub added_complete_rows: usize,
    pub final_status: KISMarketDataEvidenceSmokeFinalStatus,
    pub final_recommendation: KISMarketDataEvidenceSmokeRecommendation,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KISMarketDataEvidenceSmokeRunner;

impl Default for KISMarketDataEvidenceSmokeConfig {
    fn default() -> Self {
        Self {
            smoke_id: "sprint58-kis-market-data-smoke".to_string(),
            auth_closure_config_path: None,
            dry_run_config_path: None,
            collection_plan_v2_config_path: None,
            endpoint_policy_path: None,
            barrier_profile_registry_path: None,
            output_root: default_output_root(),
            run_auth_closure: true,
            run_dry_run: true,
            run_collection_plan: true,
            run_fixture_replay: true,
            run_local_import: true,
            run_operator_live_collection: false,
            run_schema_drift: true,
            run_canonical_validation: true,
            run_preflight: true,
            run_candle_sufficiency: true,
            run_outcome_link_closure: true,
            run_complete_row_closure: true,
            run_evidence_depth: true,
            run_trinity_loop: true,
            run_control_tower_refresh: true,
            max_bytes: default_max_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl KISMarketDataEvidenceSmokeConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.smoke_id.trim().is_empty() {
            return Err("kis market-data smoke id must not be empty".to_string());
        }
        if [
            self.auth_closure_config_path.as_deref(),
            self.dry_run_config_path.as_deref(),
            self.collection_plan_v2_config_path.as_deref(),
            self.endpoint_policy_path.as_deref(),
            self.barrier_profile_registry_path.as_deref(),
            Some(self.output_root.as_str()),
        ]
        .into_iter()
        .flatten()
        .any(|path| path.contains("://"))
        {
            return Err("kis market-data smoke paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.smoke_id)
    }
}

impl KISMarketDataEvidenceSmokeReport {
    pub fn to_text(&self) -> String {
        [
            "market_data_only_warning=kis market-data smoke is bounded, research-only, and local-first".to_string(),
            format!("smoke_id={}", self.smoke_id),
            format!("auth_closure_status={:?}", self.auth_closure_status),
            format!("dry_run_status={:?}", self.dry_run_status),
            format!("collection_plan_status={:?}", self.collection_plan_status),
            format!("schema_drift_status={}", self.schema_drift_status.clone().unwrap_or_default()),
            format!(
                "canonical_validation_status={}",
                self.canonical_validation_status.clone().unwrap_or_default()
            ),
            format!("preflight_status={}", self.preflight_status.clone().unwrap_or_default()),
            format!(
                "candle_sufficiency_status={}",
                self.candle_sufficiency_status.clone().unwrap_or_default()
            ),
            format!(
                "outcome_link_closure_status={}",
                self.outcome_link_closure_status.clone().unwrap_or_default()
            ),
            format!(
                "complete_row_closure_status={}",
                self.complete_row_closure_status.clone().unwrap_or_default()
            ),
            format!("evidence_depth_status={}", self.evidence_depth_status.clone().unwrap_or_default()),
            format!("trinity_loop_status={}", self.trinity_loop_status.clone().unwrap_or_default()),
            format!(
                "control_tower_refresh_status={}",
                self.control_tower_refresh_status.clone().unwrap_or_default()
            ),
            format!("added_canonical_csvs={}", self.added_canonical_csvs),
            format!("added_official_rows={}", self.added_official_rows),
            format!("added_preflight_ready_rows={}", self.added_preflight_ready_rows),
            format!("added_official_ready_candles={}", self.added_official_ready_candles),
            format!("added_outcome_links={}", self.added_outcome_links),
            format!("added_counterfactuals={}", self.added_counterfactuals),
            format!("added_complete_rows={}", self.added_complete_rows),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("blockers={}", self.blockers.join("|")),
            format!("warnings={}", self.warnings.join("|")),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_market_data_smoke_report.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_market_data_smoke_report.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl KISMarketDataEvidenceSmokeRunner {
    pub fn run(
        &self,
        config: &KISMarketDataEvidenceSmokeConfig,
    ) -> Result<KISMarketDataSmokeControlTowerBundle, String> {
        config.validate()?;
        let output_dir = config.artifact_dir();
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

        let auth_config = if let Some(path) = &config.auth_closure_config_path {
            KISAuthClosureConfig::from_toml_path(Path::new(path))?
        } else {
            KISAuthClosureConfig {
                closure_id: format!("{}-auth", config.smoke_id),
                output_root: config.output_root.clone(),
                ..KISAuthClosureConfig::default()
            }
        };
        let auth_closure_report = if config.run_auth_closure {
            KISAuthClosureRunner::default().run(&auth_config)?
        } else {
            KISAuthClosureRunner::default().run(&KISAuthClosureConfig {
                closure_id: format!("{}-auth-diagnostic", config.smoke_id),
                output_root: config.output_root.clone(),
                require_app_key: false,
                require_app_secret: false,
                require_base_url: false,
                ..KISAuthClosureConfig::default()
            })?
        };

        let dry_run_config = if let Some(path) = &config.dry_run_config_path {
            KISMarketDataDryRunConfig::from_toml_path(Path::new(path))?
        } else {
            KISMarketDataDryRunConfig {
                dry_run_id: format!("{}-dry-run", config.smoke_id),
                kis_auth_closure_config_path: config.auth_closure_config_path.clone(),
                endpoint_policy_path: config.endpoint_policy_path.clone(),
                output_root: config.output_root.clone(),
                ..KISMarketDataDryRunConfig::default()
            }
        };
        let dry_run_report = if config.run_dry_run {
            KISMarketDataDryRunRunner::default()
                .run_with_auth(&dry_run_config, &auth_closure_report)?
        } else {
            KISMarketDataDryRunRunner::default().run_with_auth(
                &KISMarketDataDryRunConfig {
                    dry_run_id: format!("{}-dry-run-diagnostic", config.smoke_id),
                    output_root: config.output_root.clone(),
                    ..KISMarketDataDryRunConfig::default()
                },
                &auth_closure_report,
            )?
        };

        let collection_plan_config = if let Some(path) = &config.collection_plan_v2_config_path {
            KISCollectionPlanV2Config::from_toml_path(Path::new(path))?
        } else {
            KISCollectionPlanV2Config {
                plan_id: format!("{}-collection-plan", config.smoke_id),
                dry_run_report_paths: vec![
                    dry_run_config
                        .artifact_dir()
                        .join("kis_market_data_dry_run.json")
                        .display()
                        .to_string(),
                ],
                endpoint_policy_path: config.endpoint_policy_path.clone(),
                output_root: config.output_root.clone(),
                run_fixture_replay: config.run_fixture_replay,
                run_local_import: config.run_local_import,
                run_operator_live_collection: config.run_operator_live_collection,
                ..KISCollectionPlanV2Config::default()
            }
        };
        let collection_plan_v2 = if config.run_collection_plan {
            KISCollectionPlanV2Runner::default()
                .run_with_dry_run(&collection_plan_config, Some(&dry_run_report))?
        } else {
            KISCollectionPlanV2Runner::default().run_with_dry_run(
                &KISCollectionPlanV2Config {
                    plan_id: format!("{}-collection-plan-diagnostic", config.smoke_id),
                    output_root: config.output_root.clone(),
                    ..KISCollectionPlanV2Config::default()
                },
                None,
            )?
        };

        let activation_config = derive_activation_config(
            config,
            &auth_config,
            &collection_plan_config,
            &dry_run_report,
        );
        let activation_bundle =
            KISOfficialMarketDataActivationRunner::default().run(&activation_config)?;

        let depth_bundle = if config.run_evidence_depth {
            let refresh_config_path = write_refresh_config(config, &activation_config)?;
            let depth_config = KISEvidenceDepthRunConfig {
                run_id: format!("{}-depth", config.smoke_id),
                kis_activation_report_paths: vec![
                    activation_config
                        .output_dir()
                        .join("kis_market_data_activation_report.json")
                        .display()
                        .to_string(),
                ],
                kis_candle_sufficiency_paths: vec![
                    activation_config
                        .output_dir()
                        .join("kis_candle_sufficiency.json")
                        .display()
                        .to_string(),
                ],
                kis_outcome_link_closure_paths: vec![
                    activation_config
                        .output_dir()
                        .join("kis_outcome_link_closure.json")
                        .display()
                        .to_string(),
                ],
                trinity_loop_config_paths: optional_existing_path(
                    "examples/soma_trinity_operational_loop_kis.toml",
                )
                .into_iter()
                .collect(),
                control_tower_config_paths: vec![refresh_config_path.display().to_string()],
                output_root: config.output_root.clone(),
                run_trinity_operational_loop: config.run_trinity_loop,
                run_control_tower_refresh: config.run_control_tower_refresh,
                reason_codes: vec![ReasonCode::DeterministicPath],
                ..KISEvidenceDepthRunConfig::default()
            };
            Some(KISEvidenceDepthRunRunner::default().run(&depth_config, None)?)
        } else {
            None
        };

        let environment_isolation_report =
            EnvironmentIsolationRunner::default().run(&EnvironmentIsolationConfig {
                report_id: format!("{}-env", config.smoke_id),
                output_root: config.output_root.clone(),
                ..EnvironmentIsolationConfig::default()
            })?;

        let smoke_report = build_smoke_report(
            config,
            &auth_closure_report,
            &dry_run_report,
            &collection_plan_v2,
            &activation_bundle,
            depth_bundle.as_ref(),
        );
        smoke_report.write_to_dir(&output_dir)?;

        let audit_report =
            SecretRedactionAuditRunner::default().run(&SecretRedactionAuditConfig {
                audit_id: format!("{}-audit", config.smoke_id),
                artifact_paths: collect_artifact_paths(&output_dir),
                output_root: config.output_root.clone(),
                ..SecretRedactionAuditConfig::default()
            })?;

        let auto_refresh_report =
            ControlTowerAutoRefreshRunner::default().run(&ControlTowerAutoRefreshConfig {
                refresh_id: format!("{}-auto-refresh", config.smoke_id),
                control_tower_refresh_config_path: Some(
                    write_refresh_config(config, &activation_config)?
                        .display()
                        .to_string(),
                ),
                source_smoke_report_paths: vec![
                    output_dir
                        .join("kis_market_data_smoke_report.json")
                        .display()
                        .to_string(),
                ],
                secret_redaction_audit_report_paths: vec![
                    PathBuf::from(&config.output_root)
                        .join(format!("{}-audit", config.smoke_id))
                        .join("secret_redaction_audit.json")
                        .display()
                        .to_string(),
                ],
                output_root: config.output_root.clone(),
                reason_codes: vec![ReasonCode::DeterministicPath],
                ..ControlTowerAutoRefreshConfig::default()
            })?;

        let runbook_report = OperationalRunbookV2Runner::default().run_with_reports(
            &OperationalRunbookV2Config {
                runbook_id: format!("{}-runbook", config.smoke_id),
                output_root: config.output_root.clone(),
                ..OperationalRunbookV2Config::default()
            },
            Some(&smoke_report),
            Some(&auto_refresh_report),
        )?;

        let storage_report = format!(
            "artifact_count={};max_bytes={}",
            collect_artifact_paths(&output_dir).len(),
            config.max_bytes
        );
        fs::write(output_dir.join("storage_report.txt"), &storage_report)
            .map_err(|err| err.to_string())?;
        let final_summary = format!(
            "smoke_status={:?};recommendation={:?};refresh_status={:?};runbook_status={:?}",
            smoke_report.final_status,
            smoke_report.final_recommendation,
            auto_refresh_report.refresh_status,
            runbook_report.final_status
        );
        fs::write(output_dir.join("summary.txt"), &final_summary).map_err(|err| err.to_string())?;
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::KISMarketDataSmokeBuilt);
        Ok(KISMarketDataSmokeControlTowerBundle {
            auth_closure_report,
            dry_run_report,
            collection_plan_v2,
            market_data_smoke_report: smoke_report,
            environment_isolation_report,
            secret_redaction_audit_report: audit_report,
            control_tower_auto_refresh_report: auto_refresh_report,
            operational_runbook_v2_report: runbook_report,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&reason_codes),
        })
    }
}

fn derive_activation_config(
    config: &KISMarketDataEvidenceSmokeConfig,
    auth_config: &KISAuthClosureConfig,
    plan_config: &KISCollectionPlanV2Config,
    dry_run_report: &KISMarketDataDryRunReport,
) -> KISMarketDataActivationConfig {
    KISMarketDataActivationConfig {
        activation_id: format!("{}-activation", config.smoke_id),
        local_kis_canonical_csv_paths: plan_config.local_canonical_csv_paths.clone(),
        domestic_symbol_whitelist_path: plan_config.domestic_symbol_whitelist_path.clone(),
        overseas_symbol_whitelist_path: plan_config.overseas_symbol_whitelist_path.clone(),
        endpoint_policy_path: plan_config
            .endpoint_policy_path
            .clone()
            .or(config.endpoint_policy_path.clone()),
        barrier_profile_registry_path: config.barrier_profile_registry_path.clone(),
        output_root: config.output_root.clone(),
        require_kis_app_key: auth_config.require_app_key,
        require_kis_app_secret: auth_config.require_app_secret,
        require_kis_base_url: auth_config.require_base_url,
        require_provenance: false,
        require_preflight: false,
        run_collection_dry_run: config.run_dry_run,
        run_fixture_replay: config.run_fixture_replay,
        run_local_import: config.run_local_import,
        run_live_market_data_collection: config.run_operator_live_collection
            && plan_config.run_operator_live_collection
            && dry_run_report.safe_to_run_operator_live_collection,
        run_preflight: config.run_preflight,
        run_candle_sufficiency: config.run_candle_sufficiency,
        run_outcome_link_closure: config.run_outcome_link_closure,
        reason_codes: vec![ReasonCode::DeterministicPath],
        ..KISMarketDataActivationConfig::default()
    }
}

fn write_refresh_config(
    config: &KISMarketDataEvidenceSmokeConfig,
    activation_config: &KISMarketDataActivationConfig,
) -> Result<PathBuf, String> {
    let path = config
        .artifact_dir()
        .join("control_tower_auto_refresh_input.toml");
    let refresh = crate::ui::ControlTowerRefreshConfig {
        refresh_id: format!("{}-refresh", config.smoke_id),
        control_tower_v1_config_path: optional_existing_path(
            "examples/soma_control_tower_v1_kis.toml",
        ),
        trinity_loop_report_paths: optional_existing_path(
            "examples/sprint57_data/trinity_loop_refresh.json",
        )
        .into_iter()
        .collect(),
        operational_runbook_report_paths: optional_existing_path(
            "examples/sprint57_data/operational_runbook_expected.json",
        )
        .into_iter()
        .collect(),
        output_root: config.output_root.clone(),
        kis_evidence_depth_report_paths: Vec::new(),
        owner_review_queue_paths: optional_existing_path(
            "examples/sprint54_data/owner_review_queue_sample.json",
        )
        .into_iter()
        .collect(),
        reason_codes: vec![ReasonCode::DeterministicPath],
        ..crate::ui::ControlTowerRefreshConfig::default()
    };
    fs::write(
        &path,
        toml::to_string_pretty(&refresh).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let _ = activation_config;
    Ok(path)
}

fn build_smoke_report(
    config: &KISMarketDataEvidenceSmokeConfig,
    auth_report: &KISAuthClosureReport,
    dry_run_report: &KISMarketDataDryRunReport,
    collection_plan: &KISCollectionPlanV2,
    activation_bundle: &super::kis_market_data_activation_bundle::KISOfficialMarketDataActivationBundle,
    depth_bundle: Option<&super::kis_evidence_depth::KISEvidenceDepthControlTowerBundle>,
) -> KISMarketDataEvidenceSmokeReport {
    let added_counterfactuals = activation_bundle
        .outcome_link_closure_report
        .as_ref()
        .map(|report| {
            report.generated_no_trade_counterfactuals + report.generated_risk_denied_counterfactuals
        })
        .unwrap_or_default();
    let blockers = match auth_report.closure_status {
        KISAuthClosureStatus::MissingAppKey => vec!["KIS_APP_KEY is missing".to_string()],
        KISAuthClosureStatus::MissingAppSecret => vec!["KIS_APP_SECRET is missing".to_string()],
        KISAuthClosureStatus::MissingAppKeyAndSecret => {
            vec!["KIS_APP_KEY and KIS_APP_SECRET are missing".to_string()]
        }
        KISAuthClosureStatus::MissingBaseUrl => vec!["KIS_BASE_URL is missing".to_string()],
        _ => Vec::new(),
    };
    let final_status = if matches!(
        auth_report.closure_status,
        KISAuthClosureStatus::MissingBaseUrl
    ) {
        KISMarketDataEvidenceSmokeFinalStatus::KISBaseUrlMissing
    } else if matches!(
        auth_report.closure_status,
        KISAuthClosureStatus::MissingAppKey
            | KISAuthClosureStatus::MissingAppSecret
            | KISAuthClosureStatus::MissingAppKeyAndSecret
    ) {
        KISMarketDataEvidenceSmokeFinalStatus::KISAuthMissing
    } else if matches!(
        dry_run_report.dry_run_status,
        KISMarketDataDryRunStatus::EndpointPolicyBlocked
    ) || matches!(
        collection_plan.plan_status,
        KISCollectionPlanV2Status::EndpointPolicyBlocked
    ) {
        KISMarketDataEvidenceSmokeFinalStatus::EndpointPolicyBlocked
    } else if activation_bundle
        .outcome_link_closure_report
        .as_ref()
        .is_some_and(|report| report.complete_kis_rows > 0)
    {
        KISMarketDataEvidenceSmokeFinalStatus::KISCompleteRowsImproved
    } else if added_counterfactuals > 0 {
        KISMarketDataEvidenceSmokeFinalStatus::KISCounterfactualsImproved
    } else if activation_bundle
        .outcome_link_closure_report
        .as_ref()
        .is_some_and(|report| report.generated_outcome_links > 0)
    {
        KISMarketDataEvidenceSmokeFinalStatus::KISOutcomeLinksImproved
    } else if activation_bundle.activation_report.added_kis_official_rows > 0 {
        KISMarketDataEvidenceSmokeFinalStatus::KISMarketDataEvidenceImproved
    } else if matches!(
        dry_run_report.dry_run_status,
        KISMarketDataDryRunStatus::Ready
    ) {
        KISMarketDataEvidenceSmokeFinalStatus::KISDryRunReady
    } else {
        KISMarketDataEvidenceSmokeFinalStatus::NoImprovement
    };
    let final_recommendation = match final_status {
        KISMarketDataEvidenceSmokeFinalStatus::KISAuthMissing => {
            if matches!(
                auth_report.closure_status,
                KISAuthClosureStatus::MissingAppSecret
            ) {
                KISMarketDataEvidenceSmokeRecommendation::SetKISAppSecret
            } else {
                KISMarketDataEvidenceSmokeRecommendation::SetKISAppKey
            }
        }
        KISMarketDataEvidenceSmokeFinalStatus::KISBaseUrlMissing => {
            KISMarketDataEvidenceSmokeRecommendation::SetKISBaseUrl
        }
        KISMarketDataEvidenceSmokeFinalStatus::KISDryRunReady => {
            KISMarketDataEvidenceSmokeRecommendation::RunKISCollectionPlan
        }
        KISMarketDataEvidenceSmokeFinalStatus::KISMarketDataEvidenceImproved
        | KISMarketDataEvidenceSmokeFinalStatus::KISOutcomeLinksImproved
        | KISMarketDataEvidenceSmokeFinalStatus::KISCounterfactualsImproved
        | KISMarketDataEvidenceSmokeFinalStatus::KISCompleteRowsImproved => {
            if config.run_control_tower_refresh {
                KISMarketDataEvidenceSmokeRecommendation::RunControlTowerRefresh
            } else if config.run_trinity_loop {
                KISMarketDataEvidenceSmokeRecommendation::RunTrinityOperationalLoop
            } else {
                KISMarketDataEvidenceSmokeRecommendation::RunKISEvidenceDepth
            }
        }
        _ => KISMarketDataEvidenceSmokeRecommendation::NeedMoreEvidence,
    };
    KISMarketDataEvidenceSmokeReport {
        smoke_id: config.smoke_id.clone(),
        auth_closure_status: auth_report.closure_status,
        dry_run_status: dry_run_report.dry_run_status,
        collection_plan_status: collection_plan.plan_status,
        schema_drift_status: Some(format!(
            "{:?}",
            activation_bundle
                .schema_drift_report
                .as_ref()
                .map(|report| report.schema_status)
        )),
        canonical_validation_status: Some(format!(
            "{:?}",
            activation_bundle
                .activation_report
                .canonical_batch_validation_status
        )),
        preflight_status: Some(format!(
            "{:?}",
            activation_bundle.activation_report.final_status
        )),
        candle_sufficiency_status: Some(format!(
            "{:?}",
            activation_bundle
                .activation_report
                .candle_sufficiency_status
        )),
        outcome_link_closure_status: activation_bundle
            .outcome_link_closure_report
            .as_ref()
            .map(|report| format!("{:?}", report.closure_status)),
        complete_row_closure_status: activation_bundle
            .activation_report
            .current_core_status
            .clone(),
        evidence_depth_status: depth_bundle
            .as_ref()
            .map(|bundle| format!("{:?}", bundle.kis_evidence_depth_report.depth_status)),
        trinity_loop_status: depth_bundle
            .and_then(|bundle| bundle.trinity_loop_refresh_summary.as_ref())
            .map(|summary| summary.final_status.clone()),
        control_tower_refresh_status: depth_bundle
            .map(|bundle| format!("{:?}", bundle.control_tower_refresh_report.refresh_status)),
        added_canonical_csvs: activation_bundle.activation_report.added_kis_canonical_csvs,
        added_official_rows: activation_bundle.activation_report.added_kis_official_rows,
        added_preflight_ready_rows: activation_bundle
            .activation_report
            .added_kis_preflight_ready_rows,
        added_official_ready_candles: activation_bundle
            .activation_report
            .added_kis_official_ready_candles,
        added_outcome_links: activation_bundle.activation_report.added_kis_outcome_links,
        added_counterfactuals,
        added_complete_rows: activation_bundle.activation_report.added_complete_kis_rows,
        final_status,
        final_recommendation,
        blockers,
        warnings: depth_bundle
            .map(|bundle| bundle.kis_evidence_depth_report.warnings.clone())
            .unwrap_or_default(),
        reason_codes: stable_reason_codes(
            &[
                config.reason_codes.clone(),
                vec![ReasonCode::KISMarketDataSmokeBuilt],
            ]
            .concat(),
        ),
    }
}

fn collect_artifact_paths(root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    collect_recursive(root, &mut paths);
    paths.sort();
    paths
}

fn collect_recursive(root: &Path, paths: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, paths);
            } else {
                paths.push(path.display().to_string());
            }
        }
    }
}

fn optional_existing_path(path: &str) -> Option<String> {
    let candidate = Path::new(path);
    candidate.exists().then(|| candidate.display().to_string())
}
