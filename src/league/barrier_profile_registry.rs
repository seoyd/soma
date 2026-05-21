use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::triple_barrier_reference_builder::TripleBarrierTieBreakPolicy;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BarrierProfileKind {
    #[default]
    PrimaryPreregistered,
    SecondaryPreregistered,
    Diagnostic,
    Exploratory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BarrierProfileIntendedUse {
    #[default]
    OfficialSufficiency,
    DiagnosticOnly,
    ResearchExploration,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BarrierProfile {
    pub profile_id: String,
    pub profile_kind: BarrierProfileKind,
    pub horizon_bars: usize,
    pub take_profit_pct: f64,
    pub stop_loss_pct: f64,
    pub cost_bps: f64,
    pub slippage_bps: f64,
    #[serde(default)]
    pub tie_break_policy: TripleBarrierTieBreakPolicy,
    #[serde(default)]
    pub intended_use: BarrierProfileIntendedUse,
    #[serde(default = "default_true")]
    pub registered_before_outcome_eval: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BarrierProfileRegistryConfig {
    pub registry_id: String,
    #[serde(default)]
    pub profiles: Vec<BarrierProfile>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub allow_diagnostic_profiles: bool,
    #[serde(default)]
    pub allow_exploratory_profiles: bool,
    #[serde(default = "default_true")]
    pub require_primary_profile: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BarrierProfileRegistry {
    pub registry_id: String,
    pub primary_profiles: Vec<BarrierProfile>,
    pub secondary_profiles: Vec<BarrierProfile>,
    pub diagnostic_profiles: Vec<BarrierProfile>,
    pub exploratory_profiles: Vec<BarrierProfile>,
    pub official_sufficiency_eligible_profiles: Vec<BarrierProfile>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BarrierProfileRegistryBuilder;

impl Default for BarrierProfileRegistryConfig {
    fn default() -> Self {
        Self {
            registry_id: "barrier-profile-registry".to_string(),
            profiles: vec![BarrierProfile {
                profile_id: "primary-preregistered".to_string(),
                profile_kind: BarrierProfileKind::PrimaryPreregistered,
                horizon_bars: 3,
                take_profit_pct: 0.02,
                stop_loss_pct: 0.01,
                cost_bps: 5.0,
                slippage_bps: 2.0,
                tie_break_policy: TripleBarrierTieBreakPolicy::StopFirst,
                intended_use: BarrierProfileIntendedUse::OfficialSufficiency,
                registered_before_outcome_eval: true,
                reason_codes: vec![ReasonCode::DeterministicPath],
            }],
            output_root: default_output_root(),
            allow_diagnostic_profiles: false,
            allow_exploratory_profiles: false,
            require_primary_profile: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl BarrierProfileRegistryConfig {
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
        if self.registry_id.trim().is_empty() {
            return Err("barrier profile registry id must not be empty".to_string());
        }
        if is_remote_path(&self.output_root)
            || self
                .profiles
                .iter()
                .any(|profile| profile.profile_id.contains("://"))
        {
            return Err("barrier profile registry paths must be local".to_string());
        }
        if self.profiles.is_empty() {
            return Err("barrier profile registry requires at least one profile".to_string());
        }
        let mut seen = std::collections::BTreeSet::new();
        for profile in &self.profiles {
            profile.validate()?;
            if !seen.insert(profile.profile_id.clone()) {
                return Err(format!(
                    "barrier profile id '{}' must be unique",
                    profile.profile_id
                ));
            }
            if matches!(profile.profile_kind, BarrierProfileKind::Diagnostic)
                && !self.allow_diagnostic_profiles
            {
                return Err(format!(
                    "diagnostic barrier profile '{}' is disabled by config",
                    profile.profile_id
                ));
            }
            if matches!(profile.profile_kind, BarrierProfileKind::Exploratory)
                && !self.allow_exploratory_profiles
            {
                return Err(format!(
                    "exploratory barrier profile '{}' is disabled by config",
                    profile.profile_id
                ));
            }
        }
        if self.require_primary_profile
            && !self.profiles.iter().any(|profile| {
                matches!(
                    profile.profile_kind,
                    BarrierProfileKind::PrimaryPreregistered
                )
            })
        {
            return Err(
                "barrier profile registry requires a primary preregistered profile".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.registry_id)
    }
}

impl BarrierProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.profile_id.trim().is_empty() {
            return Err("barrier profile id must not be empty".to_string());
        }
        if self.profile_id.contains("://") {
            return Err("barrier profile id must not contain remote paths".to_string());
        }
        if self.horizon_bars == 0 {
            return Err(format!(
                "barrier profile '{}' horizon_bars must be positive",
                self.profile_id
            ));
        }
        if !(0.0..=1.0).contains(&self.take_profit_pct) {
            return Err(format!(
                "barrier profile '{}' take_profit_pct must be between 0 and 1",
                self.profile_id
            ));
        }
        if !(0.0..=1.0).contains(&self.stop_loss_pct) {
            return Err(format!(
                "barrier profile '{}' stop_loss_pct must be between 0 and 1",
                self.profile_id
            ));
        }
        if self.cost_bps < 0.0 || self.slippage_bps < 0.0 {
            return Err(format!(
                "barrier profile '{}' cost/slippage must be non-negative",
                self.profile_id
            ));
        }
        Ok(())
    }

    pub fn official_sufficiency_eligible(&self) -> bool {
        matches!(
            self.profile_kind,
            BarrierProfileKind::PrimaryPreregistered | BarrierProfileKind::SecondaryPreregistered
        ) && self.registered_before_outcome_eval
    }

    pub fn diagnostic_only(&self) -> bool {
        !self.official_sufficiency_eligible()
            || !matches!(
                self.intended_use,
                BarrierProfileIntendedUse::OfficialSufficiency
            )
    }
}

impl BarrierProfileRegistryBuilder {
    pub fn build(
        &self,
        config: &BarrierProfileRegistryConfig,
    ) -> Result<BarrierProfileRegistry, String> {
        config.validate()?;
        let mut primary_profiles = Vec::new();
        let mut secondary_profiles = Vec::new();
        let mut diagnostic_profiles = Vec::new();
        let mut exploratory_profiles = Vec::new();
        let mut warnings = Vec::new();

        let mut profiles = config.profiles.clone();
        profiles.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        for profile in profiles {
            if matches!(profile.profile_kind, BarrierProfileKind::Diagnostic)
                && profile.intended_use == BarrierProfileIntendedUse::OfficialSufficiency
            {
                warnings.push(format!(
                    "diagnostic profile '{}' cannot satisfy official sufficiency and remains diagnostic-only",
                    profile.profile_id
                ));
            }
            if matches!(profile.profile_kind, BarrierProfileKind::Exploratory)
                && profile.intended_use == BarrierProfileIntendedUse::OfficialSufficiency
            {
                warnings.push(format!(
                    "exploratory profile '{}' cannot satisfy official sufficiency and remains exploratory-only",
                    profile.profile_id
                ));
            }
            if !profile.registered_before_outcome_eval {
                warnings.push(format!(
                    "profile '{}' was not registered before outcome evaluation and is excluded from official sufficiency",
                    profile.profile_id
                ));
            }
            match profile.profile_kind {
                BarrierProfileKind::PrimaryPreregistered => primary_profiles.push(profile),
                BarrierProfileKind::SecondaryPreregistered => secondary_profiles.push(profile),
                BarrierProfileKind::Diagnostic => diagnostic_profiles.push(profile),
                BarrierProfileKind::Exploratory => exploratory_profiles.push(profile),
            }
        }

        let official_sufficiency_eligible_profiles = primary_profiles
            .iter()
            .chain(secondary_profiles.iter())
            .filter(|profile| profile.official_sufficiency_eligible())
            .cloned()
            .collect::<Vec<_>>();

        Ok(BarrierProfileRegistry {
            registry_id: config.registry_id.clone(),
            primary_profiles,
            secondary_profiles,
            diagnostic_profiles,
            exploratory_profiles,
            official_sufficiency_eligible_profiles,
            warnings,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::DeterministicPath,
                        ReasonCode::LocalFileOnly,
                        ReasonCode::OfficialEvidenceCounted,
                    ])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl BarrierProfileRegistry {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.registry_id.clone()),
        )
    }

    pub fn has_primary_profile(&self) -> bool {
        !self.primary_profiles.is_empty()
    }

    pub fn official_profile(&self, profile_id: Option<&str>) -> Option<&BarrierProfile> {
        profile_id
            .and_then(|id| {
                self.official_sufficiency_eligible_profiles
                    .iter()
                    .find(|profile| profile.profile_id == id)
            })
            .or_else(|| self.official_sufficiency_eligible_profiles.first())
    }

    pub fn is_diagnostic_only(&self) -> bool {
        self.official_sufficiency_eligible_profiles.is_empty()
            && (!self.diagnostic_profiles.is_empty() || !self.exploratory_profiles.is_empty())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("registry_id={}", self.registry_id),
            format!("primary_profiles={}", self.primary_profiles.len()),
            format!("secondary_profiles={}", self.secondary_profiles.len()),
            format!("diagnostic_profiles={}", self.diagnostic_profiles.len()),
            format!("exploratory_profiles={}", self.exploratory_profiles.len()),
            format!(
                "official_sufficiency_eligible_profiles={}",
                self.official_sufficiency_eligible_profiles.len()
            ),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(
            self.primary_profiles
                .iter()
                .chain(self.secondary_profiles.iter())
                .chain(self.diagnostic_profiles.iter())
                .chain(self.exploratory_profiles.iter())
                .map(|profile| {
                    format!(
                        "profile_id={};profile_kind={:?};intended_use={:?};registered_before_outcome_eval={};official_sufficiency_eligible={};horizon_bars={};take_profit_pct={};stop_loss_pct={};tie_break_policy={:?}",
                        profile.profile_id,
                        profile.profile_kind,
                        profile.intended_use,
                        profile.registered_before_outcome_eval,
                        profile.official_sufficiency_eligible(),
                        profile.horizon_bars,
                        profile.take_profit_pct,
                        profile.stop_loss_pct,
                        profile.tie_break_policy,
                    )
                }),
        );
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
            output_dir.join("barrier_profile_registry.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("barrier_profile_registry.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_barrier_profile_registry_from_path_or_config(
    path: &str,
) -> Result<BarrierProfileRegistry, String> {
    if path.ends_with(".json") {
        BarrierProfileRegistry::from_json_path(Path::new(path))
    } else {
        BarrierProfileRegistryConfig::from_toml_path(Path::new(path))
            .and_then(|config| BarrierProfileRegistryBuilder::default().build(&config))
    }
}

fn default_output_root() -> String {
    "target/soma_barrier_profiles".to_string()
}

fn default_true() -> bool {
    true
}

fn is_remote_path(value: &str) -> bool {
    value.contains("://")
}
