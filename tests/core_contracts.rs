use soma_zero::{CoreContractRegistry, ReasonCode};

#[test]
fn core_contract_registry_can_be_constructed() {
    let registry = CoreContractRegistry::default();

    assert!(registry.contracts.contains_key("FeatureSchema"));
    assert!(registry.contracts.contains_key("PredictionSchema"));
}

#[test]
fn feature_schema_contract_check_passes_for_current_schema() {
    let registry = CoreContractRegistry::default();
    let result = registry.check("FeatureSchema", "1.0.0");

    assert!(result.compatible);
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::ContractVersionMatched)
    );
}

#[test]
fn prediction_schema_contract_check_passes_for_current_schema() {
    let registry = CoreContractRegistry::default();
    let result = registry.check("PredictionSchema", "1.0.0");

    assert!(result.compatible);
}

#[test]
fn mismatched_contract_version_is_reason_coded() {
    let registry = CoreContractRegistry::default();
    let result = registry.check("ExperimentConfig", "9.9.9");

    assert!(!result.compatible);
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::ContractVersionMismatched)
    );
}

#[test]
fn contract_registry_rendering_is_deterministic() {
    let left = CoreContractRegistry::default().report();
    let right = CoreContractRegistry::default().report();

    assert_eq!(left.to_text(), right.to_text());
}
