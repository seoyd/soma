mod common;
#[path = "support/sprint63_support.rs"]
mod sprint63_support;

use soma_zero::{ExternalPredictionEvaluationRunner, Mamba3FinLiteContractStatus};

#[test]
fn mamba3fin_lite_contract_ready_and_missing_card_block_work() {
    let valid = sprint63_support::import_config_from_example(
        "soma_mamba3fin_contract.toml",
        "contract-ready",
    );
    let contract = ExternalPredictionEvaluationRunner::default()
        .run_mamba_contract(&valid)
        .expect("run contract");
    assert_eq!(
        contract.contract_status,
        Mamba3FinLiteContractStatus::ContractReady
    );

    let missing = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_missing_model_card.toml",
        "contract-missing-card",
    );
    let contract = ExternalPredictionEvaluationRunner::default()
        .run_mamba_contract(&missing)
        .expect("run missing-card contract");
    assert_eq!(
        contract.contract_status,
        Mamba3FinLiteContractStatus::BlockedByMissingModelCard
    );
}
