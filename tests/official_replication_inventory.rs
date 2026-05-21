mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;

use serde_json::json;
use soma_zero::{
    CommitteeReferencePackRunner, OfficialCommitteeScenarioPackBuilder,
    OfficialProviderReadinessConfig, OfficialProviderReadinessRunner,
    OfficialReplicationArtifactInventory, OfficialReplicationArtifactKind, ProviderRealityConfig,
    ProviderRealityRunner, SufficiencyClosureConfig, SufficiencyClosureRunner,
};

#[test]
fn inventory_detects_supported_artifacts_and_source_boundaries() {
    let readiness =
        OfficialProviderReadinessRunner::default().run(&OfficialProviderReadinessConfig {
            report_id: "inventory-readiness".to_string(),
            output_dir: common::output_dir("inventory-readiness-out")
                .display()
                .to_string(),
            ..OfficialProviderReadinessConfig::default()
        });
    let readiness_path =
        common::output_dir("inventory-readiness-json").join("provider_readiness_report.json");
    fs::write(
        &readiness_path,
        readiness.to_json_string().expect("readiness json"),
    )
    .expect("write readiness");

    let reality = ProviderRealityRunner::default()
        .run(&ProviderRealityConfig {
            report_id: "inventory-reality".to_string(),
            output_dir: common::output_dir("inventory-reality-out")
                .display()
                .to_string(),
            ..ProviderRealityConfig::default()
        })
        .expect("reality");
    let reality_path =
        common::output_dir("inventory-reality-json").join("provider_reality_report.json");
    fs::write(
        &reality_path,
        reality.to_json_string().expect("reality json"),
    )
    .expect("write reality");

    let collection_path =
        common::output_dir("inventory-collection").join("official_collection_report.json");
    fs::write(
        &collection_path,
        serde_json::to_string_pretty(&json!({
            "plan_id": "inventory-collection",
            "entry_reports": [{
                "entry_id": "aapl",
                "provider_kind": "AlphaVantage",
                "symbol": "AAPL",
                "venue": "Nasdaq",
                "timeframe": "OneDay",
                "status": "Collected",
                "canonical_csv_path": null,
                "manifest_path": null,
                "provenance_path": null,
                "preflight_status": "ReadyForRealEvidence",
                "row_count": 10,
                "request_count": 1,
                "bytes_written": 10,
                "compressed": false,
                "ready_for_evidence": true,
                "reason_codes": ["OfficialApiCollected"]
            }],
            "storage_budget_report": {"max_bytes": 1000, "bytes_written": 10, "budget_exceeded": false, "reason_codes": []},
            "ready_entries_count": 1,
            "skipped_entries_count": 0,
            "failed_entries_count": 0,
            "official_api_collected_count": 1,
            "reason_codes": ["OfficialCollectionRan"]
        }))
        .expect("collection json"),
    )
    .expect("write collection");

    let csv_path = official_committee_support::write_official_csv_bundle(
        "inventory-aapl",
        "AAPL",
        4,
        true,
        true,
        true,
    );
    let yfinance_path = common::output_dir("inventory-yfinance").join("yfinance_report.json");
    fs::write(
        &yfinance_path,
        serde_json::to_string_pretty(&json!({"yfinance_symbols": ["AAPL"]})).expect("yf json"),
    )
    .expect("write yfinance");
    let fixture_path = common::output_dir("inventory-fixture").join("fixture_rows.json");
    fs::write(
        &fixture_path,
        serde_json::to_string_pretty(&json!({"rows": [{"symbol": "AAPL"}]})).expect("fixture json"),
    )
    .expect("write fixture");
    let crypto_path = official_committee_support::write_crypto_evidence_lane("inventory-crypto");

    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&official_committee_support::controlled_pack_config(
            "inventory-pack",
            false,
        ))
        .expect("pack");
    let pack_dir = common::output_dir("inventory-pack-store");
    let pack_path = pack.write_to_dir(&pack_dir).expect("pack write");

    let reference_bundle = CommitteeReferencePackRunner::default()
        .run(&official_committee_support::controlled_reference_pack_config("inventory-reference"))
        .expect("reference bundle");
    let reference_path = reference_bundle
        .reference_pack
        .write_to_dir(&common::output_dir("inventory-reference-store"))
        .expect("reference write");

    let closure_report = SufficiencyClosureRunner::default()
        .run_with_pack(
            &SufficiencyClosureConfig::default(),
            &reference_bundle.reference_pack,
        )
        .expect("closure");
    let closure_path = common::output_dir("inventory-closure").join("sufficiency_closure.json");
    fs::write(
        &closure_path,
        serde_json::to_string_pretty(&closure_report).expect("closure json"),
    )
    .expect("write closure");

    let coverage_path =
        common::output_dir("inventory-coverage").join("outcome_coverage_report.json");
    fs::write(
        &coverage_path,
        serde_json::to_string_pretty(&json!({
            "coverage_id": "inventory-coverage",
            "cells": [],
            "total_rows": 0,
            "official_rows": 0,
            "row_level_rows": 0,
            "summary_derived_rows": 0,
            "outcome_linked_rows": 0,
            "baseline_linked_rows": 0,
            "external_linked_rows": 0,
            "no_trade_counterfactuals": 0,
            "risk_denied_counterfactuals": 0,
            "no_lookahead_violations": 0,
            "source_summary": "",
            "coverage_status": "InsufficientCoverage",
            "reason_codes": []
        }))
        .expect("coverage json"),
    )
    .expect("write coverage");

    let inventory = OfficialReplicationArtifactInventory::from_paths(&vec![
        readiness_path.display().to_string(),
        reality_path.display().to_string(),
        collection_path.display().to_string(),
        csv_path.display().to_string(),
        csv_path
            .parent()
            .expect("csv dir")
            .join("preflight_report.json")
            .display()
            .to_string(),
        csv_path
            .parent()
            .expect("csv dir")
            .join("official_provenance.json")
            .display()
            .to_string(),
        crypto_path.display().to_string(),
        pack_path.display().to_string(),
        reference_path.display().to_string(),
        closure_path.display().to_string(),
        coverage_path.display().to_string(),
        yfinance_path.display().to_string(),
        fixture_path.display().to_string(),
    ]);

    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::ProviderReadinessReport)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::ProviderRealityReport)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::OfficialCollectionReport)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::OfficialCanonicalCsv)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::OfficialPreflightReport)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::OfficialProvenance)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::EvidenceLaneReport)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::OfficialCommitteePack)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::GeneratedReferencePack)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::SufficiencyClosureReport)
    );
    assert!(
        inventory
            .descriptors
            .iter()
            .any(|d| d.artifact_kind == OfficialReplicationArtifactKind::OutcomeCoverageReport)
    );
    assert!(inventory.descriptors.iter().any(|d| d.source_research_only));
    assert!(inventory.descriptors.iter().any(|d| d.source_fixture_only));
    assert!(inventory.descriptors.iter().any(|d| d.source_crypto_only));
}

#[test]
fn inventory_reason_codes_unknown_and_invalid_files_without_panicking_and_is_deterministic() {
    let invalid_path = common::output_dir("inventory-invalid").join("mystery.bin");
    fs::write(&invalid_path, b"not-json-not-csv").expect("write invalid");
    let first =
        OfficialReplicationArtifactInventory::from_paths(&vec![invalid_path.display().to_string()]);
    let second =
        OfficialReplicationArtifactInventory::from_paths(&vec![invalid_path.display().to_string()]);
    assert_eq!(first, second);
    assert_eq!(first.unknown_count, 1);
    assert_eq!(
        first.descriptors[0].artifact_kind,
        OfficialReplicationArtifactKind::Unknown
    );
    assert!(!first.descriptors[0].reason_codes.is_empty());
}
