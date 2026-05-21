mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::path::Path;

use serde_json::json;
use soma_zero::{
    CommitteeOutcomeLinker, CommitteeOutcomeLinkerConfig, OfficialCommitteeScenarioPackBuilder,
};

fn pack_path(name: &str) -> std::path::PathBuf {
    let cfg = official_committee_support::controlled_pack_config(name, false);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&cfg)
        .expect("pack");
    let dir = common::output_dir(&format!("{name}-pack-store"));
    pack.write_to_dir(&dir).expect("write");
    dir.join("official_scenario_pack.json")
}

#[test]
fn exact_and_tolerance_timestamp_matching_work() {
    let path = pack_path("outcome-link-exact");
    let exact = CommitteeOutcomeLinker::default()
        .link_from_config(&CommitteeOutcomeLinkerConfig {
            linker_id: "outcome-link-exact".to_string(),
            scenario_pack_path: Some(path.display().to_string()),
            outcome_artifact_paths: vec![
                official_committee_support::write_outcomes("outcome-link-exact", true)
                    .display()
                    .to_string(),
            ],
            output_root: common::output_dir("outcome-link-exact-out")
                .display()
                .to_string(),
            strict_timestamp_match: true,
            max_timestamp_tolerance_ms: 0,
            require_same_symbol: true,
            require_same_horizon: true,
            reason_codes: vec![],
            ..CommitteeOutcomeLinkerConfig::default()
        })
        .expect("exact");
    assert_eq!(exact.outcome_linked_count, 3);

    let shifted_path = common::output_dir("outcome-link-tolerance").join("outcomes.json");
    official_committee_support::write_json(
        &shifted_path,
        json!({
            "outcomes": [{
                "symbol": "AAPL",
                "timestamp_ms": 1700000000500u64,
                "horizon_bars": 24,
                "triple_barrier_label": "TakeProfit",
                "net_return_pct": 0.03,
                "cost_bps": 5.0,
                "slippage_bps": 2.0,
                "source_kind": "OfficialApiCollected",
                "no_lookahead_safe": true
            }]
        }),
    );
    let tolerance = CommitteeOutcomeLinker::default()
        .link_from_config(&CommitteeOutcomeLinkerConfig {
            linker_id: "outcome-link-tolerance".to_string(),
            scenario_pack_path: Some(path.display().to_string()),
            outcome_artifact_paths: vec![shifted_path.display().to_string()],
            output_root: common::output_dir("outcome-link-tolerance-out")
                .display()
                .to_string(),
            strict_timestamp_match: false,
            max_timestamp_tolerance_ms: 1_000,
            require_same_symbol: true,
            require_same_horizon: true,
            reason_codes: vec![],
            ..CommitteeOutcomeLinkerConfig::default()
        })
        .expect("tolerance");
    assert_eq!(tolerance.outcome_linked_count, 3);
}

#[test]
fn wrong_symbol_and_horizon_do_not_link_and_unmatched_rows_remain() {
    let path = pack_path("outcome-link-mismatch");
    let wrong_symbol = common::output_dir("outcome-link-wrong-symbol").join("outcomes.json");
    official_committee_support::write_json(
        &wrong_symbol,
        json!({
            "outcomes": [{
                "symbol": "MSFT",
                "timestamp_ms": 1700000000000u64,
                "horizon_bars": 24,
                "triple_barrier_label": "TakeProfit",
                "net_return_pct": 0.03,
                "cost_bps": 5.0,
                "slippage_bps": 2.0,
                "source_kind": "OfficialApiCollected",
                "no_lookahead_safe": true
            }]
        }),
    );
    let no_symbol_match = CommitteeOutcomeLinker::default()
        .link_from_config(&CommitteeOutcomeLinkerConfig {
            linker_id: "outcome-link-wrong-symbol".to_string(),
            scenario_pack_path: Some(path.display().to_string()),
            outcome_artifact_paths: vec![wrong_symbol.display().to_string()],
            output_root: common::output_dir("outcome-link-wrong-symbol-out")
                .display()
                .to_string(),
            strict_timestamp_match: true,
            max_timestamp_tolerance_ms: 0,
            require_same_symbol: true,
            require_same_horizon: true,
            reason_codes: vec![],
            ..CommitteeOutcomeLinkerConfig::default()
        })
        .expect("wrong symbol");
    assert_eq!(no_symbol_match.outcome_linked_count, 0);
    assert_eq!(no_symbol_match.baseline_linked_count, 3);

    let wrong_horizon = common::output_dir("outcome-link-wrong-horizon").join("outcomes.json");
    official_committee_support::write_json(
        &wrong_horizon,
        json!({
            "outcomes": [{
                "symbol": "AAPL",
                "timestamp_ms": 1700000000000u64,
                "horizon_bars": 6,
                "triple_barrier_label": "TakeProfit",
                "net_return_pct": 0.03,
                "cost_bps": 5.0,
                "slippage_bps": 2.0,
                "source_kind": "OfficialApiCollected",
                "no_lookahead_safe": true
            }]
        }),
    );
    let no_horizon_match = CommitteeOutcomeLinker::default()
        .link_from_config(&CommitteeOutcomeLinkerConfig {
            linker_id: "outcome-link-wrong-horizon".to_string(),
            scenario_pack_path: Some(path.display().to_string()),
            outcome_artifact_paths: vec![wrong_horizon.display().to_string()],
            output_root: common::output_dir("outcome-link-wrong-horizon-out")
                .display()
                .to_string(),
            strict_timestamp_match: true,
            max_timestamp_tolerance_ms: 0,
            require_same_symbol: true,
            require_same_horizon: true,
            reason_codes: vec![],
            ..CommitteeOutcomeLinkerConfig::default()
        })
        .expect("wrong horizon");
    assert_eq!(no_horizon_match.outcome_linked_count, 0);
}

#[test]
fn invalid_external_schema_is_rejected_and_link_summary_is_deterministic() {
    let path = pack_path("outcome-link-external");
    let cfg = CommitteeOutcomeLinkerConfig {
        linker_id: "outcome-link-external".to_string(),
        scenario_pack_path: Some(path.display().to_string()),
        outcome_artifact_paths: vec![
            official_committee_support::write_outcomes("outcome-link-external", true)
                .display()
                .to_string(),
        ],
        baseline_artifact_paths: vec![
            official_committee_support::write_baselines("outcome-link-external")
                .display()
                .to_string(),
        ],
        external_prediction_paths: vec![
            official_committee_support::write_externals("outcome-link-external", false)
                .display()
                .to_string(),
        ],
        output_root: common::output_dir("outcome-link-external-out")
            .display()
            .to_string(),
        strict_timestamp_match: true,
        max_timestamp_tolerance_ms: 0,
        require_same_symbol: true,
        require_same_horizon: true,
        reason_codes: vec![],
    };
    official_committee_support::write_json(
        &std::path::PathBuf::from(&cfg.external_prediction_paths[0]),
        json!({
            "predictions": [{
                "symbol": "AAPL",
                "timestamp_ms": 1700000000000u64,
                "horizon_bars": 24
            }]
        }),
    );
    let first = CommitteeOutcomeLinker::default()
        .link_from_config(&cfg)
        .expect("first");
    let second = CommitteeOutcomeLinker::default()
        .link_from_config(&cfg)
        .expect("second");
    assert_eq!(first.external_linked_count, 0);
    assert_eq!(first.link_summary.to_text(), second.link_summary.to_text());
    assert!(Path::new(&cfg.output_root).is_absolute() || !cfg.output_root.contains("://"));
}
