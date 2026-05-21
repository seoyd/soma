mod common;

use std::fs;

use soma_zero::{EvidenceStore, EvidenceStoreConfig, ResearchCampaignRunner};

#[test]
fn evidence_fingerprint_is_deterministic_and_content_sensitive() {
    let a = EvidenceStore::compute_fingerprint("alpha");
    let a_again = EvidenceStore::compute_fingerprint("alpha");
    let b = EvidenceStore::compute_fingerprint("beta");
    assert_eq!(a, a_again);
    assert_ne!(a, b);
}

#[test]
fn evidence_store_lists_snapshots_deterministically_and_uses_config_timestamp() {
    let matrix = common::batch_matrix(
        "evidence-matrix",
        vec![common::dataset_entry(
            "valid",
            "generic_ohlcv_valid.csv",
            true,
        )],
        vec![common::baseline_variant("baseline_5m", true)],
    );
    let matrix_path = common::output_dir("evidence-matrix").join("matrix.toml");
    fs::write(&matrix_path, matrix.to_toml_string().expect("matrix toml")).expect("write matrix");
    let config =
        common::campaign_config("evidence-campaign", vec![matrix_path.display().to_string()]);

    let report = ResearchCampaignRunner::default().run_campaign(&config);
    assert!(report.errors.is_empty());

    let store = EvidenceStore;
    let snapshots = store
        .list_snapshots(&EvidenceStoreConfig {
            root_path: config.evidence_store_path.clone(),
            campaign_id: config.campaign_id.clone(),
            allow_overwrite: true,
            created_at_ms: None,
            reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
        })
        .expect("list snapshots");
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].created_at_ms, Some(42));
    let summary = store
        .load_snapshot_summary(std::path::Path::new(&snapshots[0].summary_path))
        .expect("load summary");
    assert!(summary.contains("campaign_id=evidence-campaign"));
}

#[test]
fn evidence_store_rejects_remote_paths() {
    let store = EvidenceStore;
    let result = store.list_snapshots(&EvidenceStoreConfig {
        root_path: "https://example.com/evidence".to_string(),
        campaign_id: "bad".to_string(),
        allow_overwrite: true,
        created_at_ms: None,
        reason_codes: vec![],
    });
    assert!(result.is_err());
}
