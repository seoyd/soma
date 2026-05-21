use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};

fn default_true() -> bool {
    true
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum ProviderPriorityMode {
    KISPrimary,
    KRXReference,
    AlphaVantageFallback,
    YFinanceResearchOnly,
    UpbitCryptoOnly,
    #[default]
    Disabled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSimplificationSelection {
    pub provider_label: String,
    pub priority_mode: ProviderPriorityMode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPriorityChange {
    pub market: String,
    pub provider_label: String,
    pub priority_mode: ProviderPriorityMode,
    pub note: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderSimplificationFinalStatus {
    KISPrimarySimplified,
    KISPrimaryBlockedByAuth,
    KISPrimaryBlockedByEndpointPolicy,
    KISPrimaryWithKRXReference,
    NeedProviderAuth,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderSimplificationConfig {
    pub simplification_id: String,
    #[serde(default)]
    pub provider_catalog_paths: Vec<String>,
    #[serde(default)]
    pub provider_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub krx_activation_report_paths: Vec<String>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub make_kis_primary_for_korean_equity: bool,
    #[serde(default = "default_true")]
    pub make_kis_primary_for_us_equity: bool,
    #[serde(default = "default_true")]
    pub retain_krx_as_reference: bool,
    #[serde(default = "default_true")]
    pub retain_alpha_vantage_as_fallback: bool,
    #[serde(default = "default_true")]
    pub retain_yfinance_as_research_only: bool,
    #[serde(default = "default_true")]
    pub retain_upbit_crypto_optional: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ProviderSimplificationConfig {
    fn default() -> Self {
        Self {
            simplification_id: "provider_simplification".to_string(),
            provider_catalog_paths: Vec::new(),
            provider_readiness_report_paths: Vec::new(),
            kis_activation_report_paths: Vec::new(),
            krx_activation_report_paths: Vec::new(),
            output_root: "target/soma_provider_simplification".to_string(),
            make_kis_primary_for_korean_equity: true,
            make_kis_primary_for_us_equity: true,
            retain_krx_as_reference: true,
            retain_alpha_vantage_as_fallback: true,
            retain_yfinance_as_research_only: true,
            retain_upbit_crypto_optional: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ProviderSimplificationConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let remote = self.output_root.contains("://")
            || self
                .provider_catalog_paths
                .iter()
                .chain(self.provider_readiness_report_paths.iter())
                .chain(self.kis_activation_report_paths.iter())
                .chain(self.krx_activation_report_paths.iter())
                .any(|path| path.contains("://"));
        if remote {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::RemotePathRejected,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn artifact_dir(&self) -> PathBuf {
        Path::new(&self.output_root).join(&self.simplification_id)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderSimplificationReport {
    pub simplification_id: String,
    pub korean_equity_primary: ProviderSimplificationSelection,
    pub us_equity_primary: ProviderSimplificationSelection,
    pub fallback_providers: Vec<String>,
    pub research_only_providers: Vec<String>,
    pub disabled_providers: Vec<String>,
    pub changed_priorities: Vec<ProviderPriorityChange>,
    pub operator_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub final_status: ProviderSimplificationFinalStatus,
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

impl ProviderSimplificationReport {
    pub fn with_fingerprint(mut self) -> Self {
        self.fingerprint = String::new();
        let material = serde_json::to_string(&self).unwrap_or_else(|_| self.to_text());
        self.fingerprint = stable_hash_string(&material);
        self
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=provider simplification is research-only and market-data-only"
                .to_string(),
            "safety_warning=no broker order account balance holdings or live trading surfaces are enabled"
                .to_string(),
            format!("simplification_id={}", self.simplification_id),
            format!(
                "korean_equity_primary={} ({:?})",
                self.korean_equity_primary.provider_label, self.korean_equity_primary.priority_mode
            ),
            format!(
                "us_equity_primary={} ({:?})",
                self.us_equity_primary.provider_label, self.us_equity_primary.priority_mode
            ),
            format!("fallback_providers={}", self.fallback_providers.join("|")),
            format!(
                "research_only_providers={}",
                self.research_only_providers.join("|")
            ),
            format!("disabled_providers={}", self.disabled_providers.join("|")),
            format!("final_status={:?}", self.final_status),
            format!("warnings={}", self.warnings.join("|")),
            format!("operator_actions={}", self.operator_actions.join("|")),
            format!("fingerprint={}", self.fingerprint),
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

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("provider_simplification_report.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("provider_simplification_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProviderSimplificationRunner;

impl ProviderSimplificationRunner {
    pub fn run(&self, config: &ProviderSimplificationConfig) -> ProviderSimplificationReport {
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::ProviderSimplificationBuilt);

        let mut warnings = Vec::new();
        let mut operator_actions = vec![
            "Keep KIS limited to market-data-only endpoints.".to_string(),
            "Keep no live trading, broker, order, account, balance, holdings, or position APIs enabled."
                .to_string(),
            "Treat KIS priority as operational simplification, not performance proof.".to_string(),
        ];

        let mut loaded_reports = Vec::new();
        let mut loaded_kis = Vec::new();
        let mut loaded_krx = Vec::new();

        load_values(
            &config.provider_readiness_report_paths,
            "provider readiness",
            &mut loaded_reports,
            &mut warnings,
            &mut reason_codes,
        );
        load_values(
            &config.kis_activation_report_paths,
            "kis activation",
            &mut loaded_kis,
            &mut warnings,
            &mut reason_codes,
        );
        load_values(
            &config.krx_activation_report_paths,
            "krx activation",
            &mut loaded_krx,
            &mut warnings,
            &mut reason_codes,
        );

        let auth_ready = detect_auth_ready(&loaded_reports, &loaded_kis);
        let endpoint_blocked = detect_endpoint_blocked(&loaded_kis);

        if !auth_ready {
            warnings.push(
                "KIS auth readiness is missing or incomplete; env values stay hidden and market-data activation stays conservative."
                    .to_string(),
            );
            operator_actions.push(
                "Provide KIS env vars locally and rerun KIS market-data-only readiness checks."
                    .to_string(),
            );
            reason_codes.push(ReasonCode::ProviderSimplificationAuthBlocked);
        }
        if endpoint_blocked {
            warnings.push(
                "Unsafe KIS endpoint usage was detected; broker/order/account surfaces remain blocked."
                    .to_string(),
            );
            operator_actions.push(
                "Remove non-market-data KIS endpoints before promoting KIS as the default provider."
                    .to_string(),
            );
            reason_codes.push(ReasonCode::ProviderSimplificationEndpointBlocked);
        }
        if config.retain_krx_as_reference {
            operator_actions.push(
                "Retain KRX as the Korean equity reference/fallback lane for validation and continuity."
                    .to_string(),
            );
        }
        if config.retain_yfinance_as_research_only {
            operator_actions.push(
                "Use yfinance only for research, diagnostics, and non-official comparison."
                    .to_string(),
            );
        }

        let mut fallback_providers = Vec::new();
        if config.retain_krx_as_reference {
            fallback_providers.push("KRX".to_string());
        }
        if config.retain_alpha_vantage_as_fallback {
            fallback_providers.push("AlphaVantage".to_string());
        }
        let mut research_only_providers = Vec::new();
        if config.retain_yfinance_as_research_only {
            research_only_providers.push("yfinance".to_string());
        }
        let mut disabled_providers = vec!["broker/order/account endpoints".to_string()];
        if !config.retain_upbit_crypto_optional {
            disabled_providers.push("Upbit".to_string());
        }

        let mut changed_priorities = Vec::new();
        if config.make_kis_primary_for_korean_equity {
            changed_priorities.push(ProviderPriorityChange {
                market: "korean_equity".to_string(),
                provider_label: "KIS".to_string(),
                priority_mode: ProviderPriorityMode::KISPrimary,
                note: "Operational primary for Korean equity market data.".to_string(),
            });
        }
        if config.make_kis_primary_for_us_equity {
            changed_priorities.push(ProviderPriorityChange {
                market: "us_equity".to_string(),
                provider_label: "KIS".to_string(),
                priority_mode: ProviderPriorityMode::KISPrimary,
                note: "Operational primary for US equity market data.".to_string(),
            });
        }
        if config.retain_krx_as_reference {
            changed_priorities.push(ProviderPriorityChange {
                market: "korean_equity".to_string(),
                provider_label: "KRX".to_string(),
                priority_mode: ProviderPriorityMode::KRXReference,
                note: "Reference and fallback lane retained.".to_string(),
            });
        }
        if config.retain_alpha_vantage_as_fallback {
            changed_priorities.push(ProviderPriorityChange {
                market: "us_equity".to_string(),
                provider_label: "AlphaVantage".to_string(),
                priority_mode: ProviderPriorityMode::AlphaVantageFallback,
                note: "Fallback lane retained for bounded research-only use.".to_string(),
            });
        }
        if config.retain_yfinance_as_research_only {
            changed_priorities.push(ProviderPriorityChange {
                market: "research".to_string(),
                provider_label: "yfinance".to_string(),
                priority_mode: ProviderPriorityMode::YFinanceResearchOnly,
                note: "Research-only comparison lane retained.".to_string(),
            });
        }
        if config.retain_upbit_crypto_optional {
            changed_priorities.push(ProviderPriorityChange {
                market: "crypto".to_string(),
                provider_label: "Upbit".to_string(),
                priority_mode: ProviderPriorityMode::UpbitCryptoOnly,
                note: "Optional crypto-only lane retained outside equity evidence.".to_string(),
            });
        }

        fallback_providers = stable_ordered_strings(&fallback_providers);
        research_only_providers = stable_ordered_strings(&research_only_providers);
        disabled_providers = stable_ordered_strings(&disabled_providers);
        warnings = stable_ordered_strings(&warnings);
        operator_actions = stable_ordered_strings(&operator_actions);
        changed_priorities.sort_by(|left, right| {
            left.market
                .cmp(&right.market)
                .then_with(|| left.provider_label.cmp(&right.provider_label))
                .then_with(|| left.note.cmp(&right.note))
        });

        let final_status = if !config.make_kis_primary_for_korean_equity
            && !config.make_kis_primary_for_us_equity
        {
            ProviderSimplificationFinalStatus::DiagnosticOnly
        } else if endpoint_blocked {
            ProviderSimplificationFinalStatus::KISPrimaryBlockedByEndpointPolicy
        } else if !auth_ready && loaded_reports.is_empty() && loaded_kis.is_empty() {
            ProviderSimplificationFinalStatus::NeedProviderAuth
        } else if !auth_ready {
            ProviderSimplificationFinalStatus::KISPrimaryBlockedByAuth
        } else if config.retain_krx_as_reference {
            ProviderSimplificationFinalStatus::KISPrimaryWithKRXReference
        } else {
            ProviderSimplificationFinalStatus::KISPrimarySimplified
        };

        ProviderSimplificationReport {
            simplification_id: config.simplification_id.clone(),
            korean_equity_primary: ProviderSimplificationSelection {
                provider_label: if config.make_kis_primary_for_korean_equity {
                    "KIS".to_string()
                } else {
                    "Disabled".to_string()
                },
                priority_mode: if config.make_kis_primary_for_korean_equity {
                    ProviderPriorityMode::KISPrimary
                } else {
                    ProviderPriorityMode::Disabled
                },
            },
            us_equity_primary: ProviderSimplificationSelection {
                provider_label: if config.make_kis_primary_for_us_equity {
                    "KIS".to_string()
                } else {
                    "Disabled".to_string()
                },
                priority_mode: if config.make_kis_primary_for_us_equity {
                    ProviderPriorityMode::KISPrimary
                } else {
                    ProviderPriorityMode::Disabled
                },
            },
            fallback_providers,
            research_only_providers,
            disabled_providers,
            changed_priorities,
            operator_actions,
            warnings,
            final_status,
            reason_codes: stable_reason_codes(&reason_codes),
            fingerprint: String::new(),
        }
        .with_fingerprint()
    }
}

pub fn provider_simplification_report_to_text(report: &ProviderSimplificationReport) -> String {
    report.to_text()
}

fn load_values(
    paths: &[String],
    label: &str,
    output: &mut Vec<Value>,
    warnings: &mut Vec<String>,
    reason_codes: &mut Vec<ReasonCode>,
) {
    for path in stable_ordered_strings(paths) {
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => output.push(value),
                Err(err) => {
                    warnings.push(format!("failed to parse {label} report at {path}: {err}"));
                    reason_codes.push(ReasonCode::DataLoadFailed);
                }
            },
            Err(_) => {
                warnings.push(format!("missing {label} report at {path}"));
                reason_codes.push(ReasonCode::MissingFile);
                reason_codes.push(ReasonCode::DashboardReportMissing);
            }
        }
    }
}

fn detect_auth_ready(readiness_reports: &[Value], kis_reports: &[Value]) -> bool {
    readiness_reports
        .iter()
        .chain(kis_reports.iter())
        .any(|value| {
            bool_field(
                value,
                &[
                    "auth_ready",
                    "safe_to_collect_rest_market_data",
                    "rest_ready",
                ],
            )
            .unwrap_or(false)
                || string_field(value, &["readiness_status", "final_status"]).is_some_and(
                    |status| {
                        matches!(
                            status.as_str(),
                            "Ready" | "KISPrimarySimplified" | "KISPrimaryWithKRXReference"
                        )
                    },
                )
        })
}

fn detect_endpoint_blocked(kis_reports: &[Value]) -> bool {
    kis_reports.iter().any(|value| {
        string_field(value, &["endpoint_policy_status", "policy_status"]).is_some_and(|status| {
            let normalized = status.to_ascii_lowercase();
            normalized.contains("block")
                || normalized.contains("deny")
                || normalized.contains("unsafe")
        }) || array_string_field(value, &["reason_codes"])
            .iter()
            .any(|reason| {
                matches!(
                    reason.as_str(),
                    "KISEndpointDenied"
                        | "KISBrokerEndpointDetected"
                        | "ProviderSimplificationEndpointBlocked"
                )
            })
    })
}

fn bool_field(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(|item| item.as_bool()))
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(|item| item.as_str())
            .map(|item| item.to_string())
    })
}

fn array_string_field(value: &Value, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .find_map(|key| {
            value.get(key).and_then(|item| {
                item.as_array().map(|items| {
                    items
                        .iter()
                        .filter_map(|entry| entry.as_str().map(|item| item.to_string()))
                        .collect::<Vec<_>>()
                })
            })
        })
        .unwrap_or_default()
}
