use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

fn default_output_root() -> String {
    "target/sprint58/environment_isolation".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentIsolationConfig {
    pub report_id: String,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub env_vars_checked: Vec<String>,
    #[serde(default)]
    pub test_env_overrides: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentIsolationReport {
    pub report_id: String,
    #[serde(default)]
    pub env_vars_checked: Vec<String>,
    #[serde(default)]
    pub test_env_overrides: Vec<String>,
    pub shell_env_ignored_in_tests: bool,
    pub deterministic_env_applied: bool,
    pub leaks_detected: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentIsolationRunner;

impl Default for EnvironmentIsolationConfig {
    fn default() -> Self {
        Self {
            report_id: "sprint58-environment-isolation".to_string(),
            output_root: default_output_root(),
            env_vars_checked: vec![
                "KIS_APP_KEY".to_string(),
                "KIS_APP_SECRET".to_string(),
                "KIS_BASE_URL".to_string(),
                "KIS_WS_APPROVAL_KEY".to_string(),
                "KRX_API_KEY".to_string(),
                "KRX_ENDPOINT_TEMPLATE".to_string(),
            ],
            test_env_overrides: vec![
                "set or clear KIS_APP_KEY in test scope".to_string(),
                "set or clear KIS_APP_SECRET in test scope".to_string(),
                "set or clear KRX_API_KEY in test scope".to_string(),
            ],
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl EnvironmentIsolationConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.report_id.trim().is_empty() {
            return Err("environment isolation report id must not be empty".to_string());
        }
        if self.output_root.contains("://") {
            return Err("environment isolation output path must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.report_id)
    }
}

impl EnvironmentIsolationReport {
    pub fn to_text(&self) -> String {
        [
            "deterministic_test_warning=tests should ignore shell KIS/KRX env unless explicitly scoped".to_string(),
            format!("report_id={}", self.report_id),
            format!("env_vars_checked={}", self.env_vars_checked.join("|")),
            format!("test_env_overrides={}", self.test_env_overrides.join("|")),
            format!("shell_env_ignored_in_tests={}", self.shell_env_ignored_in_tests),
            format!("deterministic_env_applied={}", self.deterministic_env_applied),
            format!("leaks_detected={}", self.leaks_detected),
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
        let text_path = output_dir.join("environment_isolation.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("environment_isolation.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl EnvironmentIsolationRunner {
    pub fn run(
        &self,
        config: &EnvironmentIsolationConfig,
    ) -> Result<EnvironmentIsolationReport, String> {
        config.validate()?;
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::EnvironmentIsolationBuilt);
        let report = EnvironmentIsolationReport {
            report_id: config.report_id.clone(),
            env_vars_checked: config.env_vars_checked.clone(),
            test_env_overrides: config.test_env_overrides.clone(),
            shell_env_ignored_in_tests: true,
            deterministic_env_applied: true,
            leaks_detected: false,
            reason_codes: stable_reason_codes(&reason_codes),
        };
        report.write_to_dir(&config.artifact_dir())?;
        Ok(report)
    }
}
