use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractVersion {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub compatible_from: Option<String>,
    pub breaking_change: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractCheckResult {
    pub contract_name: String,
    pub expected_version: String,
    pub actual_version: String,
    pub compatible: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreContractRegistry {
    pub contracts: BTreeMap<String, ContractVersion>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreContractRegistryReport {
    pub contracts: Vec<ContractVersion>,
    pub checks: Vec<ContractCheckResult>,
    pub fingerprint: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CoreContractRegistry {
    fn default() -> Self {
        let mut contracts = BTreeMap::new();
        for name in [
            "AuditEventSchema",
            "ChairConfig",
            "DatasetSchema",
            "ExperimentConfig",
            "FeatureSchema",
            "Mamba3FinCandidateSpec",
            "OfficialAiBenchmarkConfig",
            "OfficialCollectionPlan",
            "PredictionSchema",
            "ReasonCodeSet",
            "RiskGovernorConfig",
            "SequenceDatasetSpec",
        ] {
            contracts.insert(
                name.to_string(),
                ContractVersion {
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    compatible_from: Some("1.0.0".to_string()),
                    breaking_change: false,
                    reason_codes: vec![ReasonCode::ContractRegistryBuilt],
                },
            );
        }
        Self {
            contracts,
            reason_codes: vec![ReasonCode::ContractRegistryBuilt],
        }
    }
}

impl CoreContractRegistry {
    pub fn check(&self, contract_name: &str, expected_version: &str) -> ContractCheckResult {
        let actual = self
            .contracts
            .get(contract_name)
            .map(|item| item.version.clone())
            .unwrap_or_else(|| "missing".to_string());
        let compatible = self.contracts.get(contract_name).is_some_and(|item| {
            item.version == expected_version
                || (!item.breaking_change
                    && item.compatible_from.as_deref() == Some(expected_version))
        });
        ContractCheckResult {
            contract_name: contract_name.to_string(),
            expected_version: expected_version.to_string(),
            actual_version: actual,
            compatible,
            reason_codes: vec![if compatible {
                ReasonCode::ContractVersionMatched
            } else {
                ReasonCode::ContractVersionMismatched
            }],
        }
    }

    pub fn report(&self) -> CoreContractRegistryReport {
        let contracts = self.contracts.values().cloned().collect::<Vec<_>>();
        let checks = contracts
            .iter()
            .map(|item| self.check(&item.name, &item.version))
            .collect::<Vec<_>>();
        CoreContractRegistryReport::new(contracts, checks)
    }
}

impl CoreContractRegistryReport {
    pub fn new(contracts: Vec<ContractVersion>, checks: Vec<ContractCheckResult>) -> Self {
        let fingerprint = stable_hash_string(&format!(
            "{}|{}",
            contracts
                .iter()
                .map(|item| format!("{}:{}", item.name, item.version))
                .collect::<Vec<_>>()
                .join("|"),
            checks
                .iter()
                .map(|item| format!(
                    "{}:{}:{}:{}",
                    item.contract_name, item.expected_version, item.actual_version, item.compatible
                ))
                .collect::<Vec<_>>()
                .join("|")
        ));
        Self {
            contracts,
            checks,
            fingerprint,
            reason_codes: vec![ReasonCode::ContractRegistryBuilt],
        }
    }

    pub fn has_incompatible(&self) -> bool {
        self.checks.iter().any(|item| !item.compatible)
    }

    pub fn to_text(&self) -> String {
        let mut lines = Vec::new();
        for item in &self.contracts {
            lines.push(format!("contract={}:{}", item.name, item.version));
        }
        for check in &self.checks {
            lines.push(format!(
                "check={}:{}:{}:{}",
                check.contract_name, check.expected_version, check.actual_version, check.compatible
            ));
        }
        lines.push(format!("fingerprint={}", self.fingerprint));
        lines.join("\n")
    }
}
