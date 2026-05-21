mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::{ModelVersionRollupKey, ModelVersionSummaryStatus};

#[test]
fn rollup_key_is_deterministic_and_blocks_missing_identity() {
    let key_a = ModelVersionRollupKey {
        model_id: "ext-model-a".to_string(),
        model_version: "1.1.0".to_string(),
        model_family: Some("family".to_string()),
        dataset_fingerprint: Some("dataset".to_string()),
        feature_schema_hash: Some("feature".to_string()),
        label_manifest_hash: Some("label".to_string()),
        split_policy: Some("chrono".to_string()),
        prediction_schema_version: Some("v2".to_string()),
    };
    let key_b = key_a.clone();
    assert_eq!(key_a, key_b);

    let mut missing = key_a.clone();
    missing.model_id.clear();
    assert!(missing.validate(true, true).is_err());
    missing.model_id = "ext-model-a".to_string();
    missing.model_version.clear();
    assert!(missing.validate(true, true).is_err());
}

#[test]
fn rollup_key_contract_mismatch_creates_distinct_group() {
    let left = ModelVersionRollupKey {
        model_id: "ext-model-a".to_string(),
        model_version: "1.1.0".to_string(),
        model_family: Some("family".to_string()),
        dataset_fingerprint: Some("dataset".to_string()),
        feature_schema_hash: Some("feature-a".to_string()),
        label_manifest_hash: Some("label".to_string()),
        split_policy: Some("chrono".to_string()),
        prediction_schema_version: Some("v2".to_string()),
    };
    let mut right = left.clone();
    right.feature_schema_hash = Some("feature-b".to_string());
    assert_ne!(left, right);
    assert!(left < right || right < left);
}

#[test]
fn summary_cards_expose_conservative_statuses_and_forbidden_actions() {
    let bundle = sprint67_support::run_rollup("soma_model_ops_rollup.toml", "summary-cards");
    let cards = bundle.model_version_summary_cards;

    let keep = cards
        .iter()
        .find(|card| card.display_name == "ext-model-a:1.1.0")
        .expect("ext-model-a 1.1.0 card");
    assert_eq!(
        keep.recommended_action,
        ModelVersionSummaryStatus::KeepResearchCandidate
    );

    let retire = cards
        .iter()
        .find(|card| card.display_name == "ext-model-a:1.0.0")
        .expect("ext-model-a 1.0.0 card");
    assert_eq!(
        retire.recommended_action,
        ModelVersionSummaryStatus::RetireModelVersion
    );

    let request_more = cards
        .iter()
        .find(|card| card.display_name == "ext-model-b:1.0.0")
        .expect("ext-model-b 1.0.0 card");
    assert_eq!(
        request_more.recommended_action,
        ModelVersionSummaryStatus::RequestMorePredictions
    );
    for forbidden in ["train", "live", "runtime", "broker", "order", "account"] {
        assert!(
            request_more
                .forbidden_actions
                .iter()
                .any(|item| item == forbidden)
        );
    }
}
