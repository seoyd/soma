mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeOfficialBenchmarkFinalStatus, CommitteeOfficialBenchmarkRunner,
    CommitteeOutcomeLinkerConfig, OfficialCommitteeScenarioPackConfig,
};

#[test]
fn no_official_rows_and_no_outcomes_are_conservative() {
    let empty_pack_path = official_committee_support::write_pack_config(
        "official-benchmark-empty",
        &OfficialCommitteeScenarioPackConfig {
            pack_id: "official-benchmark-empty-pack".to_string(),
            output_root: common::output_dir("official-benchmark-empty-pack-out")
                .display()
                .to_string(),
            ..OfficialCommitteeScenarioPackConfig::default()
        },
    );
    let empty = CommitteeOfficialBenchmarkRunner::default()
        .run(&official_committee_support::controlled_benchmark_config(
            "official-benchmark-empty",
            &empty_pack_path,
            &official_committee_support::write_linker_config(
                "official-benchmark-empty",
                &CommitteeOutcomeLinkerConfig {
                    linker_id: "official-benchmark-empty-linker".to_string(),
                    scenario_pack_path: Some(
                        common::output_dir("official-benchmark-empty-pack-store")
                            .join("official_scenario_pack.json")
                            .display()
                            .to_string(),
                    ),
                    output_root: common::output_dir("official-benchmark-empty-linker-out")
                        .display()
                        .to_string(),
                    reason_codes: vec![],
                    ..CommitteeOutcomeLinkerConfig::default()
                },
            ),
            false,
        ))
        .expect("empty");
    assert_eq!(
        empty.final_status,
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreOfficialRows
    );

    let pack_cfg =
        official_committee_support::controlled_pack_config("official-benchmark-no-outcome", false);
    let pack_cfg_path =
        official_committee_support::write_pack_config("official-benchmark-no-outcome", &pack_cfg);
    let pack = soma_zero::OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_cfg)
        .expect("pack");
    let pack_dir = common::output_dir("official-benchmark-no-outcome-pack-store");
    pack.write_to_dir(&pack_dir).expect("write pack");
    let linker_cfg_path = official_committee_support::write_linker_config(
        "official-benchmark-no-outcome",
        &CommitteeOutcomeLinkerConfig {
            linker_id: "official-benchmark-no-outcome-linker".to_string(),
            scenario_pack_path: Some(
                pack_dir
                    .join("official_scenario_pack.json")
                    .display()
                    .to_string(),
            ),
            output_root: common::output_dir("official-benchmark-no-outcome-linker-out")
                .display()
                .to_string(),
            reason_codes: vec![],
            ..CommitteeOutcomeLinkerConfig::default()
        },
    );
    let no_outcome = CommitteeOfficialBenchmarkRunner::default()
        .run(&official_committee_support::controlled_benchmark_config(
            "official-benchmark-no-outcome",
            &pack_cfg_path,
            &linker_cfg_path,
            false,
        ))
        .expect("no outcome");
    assert_eq!(
        no_outcome.final_status,
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreOutcomeLinks
    );
}

#[test]
fn yfinance_fixture_and_crypto_only_map_conservatively() {
    let yfinance_cfg_path = official_committee_support::write_pack_config(
        "official-benchmark-yfinance",
        &official_committee_support::yfinance_pack_config("official-benchmark-yfinance"),
    );
    let yfinance = CommitteeOfficialBenchmarkRunner::default()
        .run(&soma_zero::CommitteeOfficialBenchmarkConfig {
            benchmark_id: "official-benchmark-yfinance".to_string(),
            scenario_pack_config_path: Some(yfinance_cfg_path.display().to_string()),
            output_root: common::output_dir("official-benchmark-yfinance-out")
                .display()
                .to_string(),
            require_core_check: false,
            reason_codes: vec![],
            ..soma_zero::CommitteeOfficialBenchmarkConfig::default()
        })
        .expect("yfinance");
    assert_eq!(
        yfinance.final_status,
        CommitteeOfficialBenchmarkFinalStatus::ResearchOnly
    );

    let fixture_cfg_path = official_committee_support::write_pack_config(
        "official-benchmark-fixture",
        &official_committee_support::fixture_pack_config("official-benchmark-fixture"),
    );
    let fixture = CommitteeOfficialBenchmarkRunner::default()
        .run(&soma_zero::CommitteeOfficialBenchmarkConfig {
            benchmark_id: "official-benchmark-fixture".to_string(),
            scenario_pack_config_path: Some(fixture_cfg_path.display().to_string()),
            output_root: common::output_dir("official-benchmark-fixture-out")
                .display()
                .to_string(),
            require_core_check: false,
            reason_codes: vec![],
            ..soma_zero::CommitteeOfficialBenchmarkConfig::default()
        })
        .expect("fixture");
    assert_eq!(
        fixture.final_status,
        CommitteeOfficialBenchmarkFinalStatus::FixtureOnly
    );

    let crypto_pack_cfg =
        official_committee_support::crypto_pack_config("official-benchmark-crypto");
    let crypto_pack_cfg_path = official_committee_support::write_pack_config(
        "official-benchmark-crypto",
        &crypto_pack_cfg,
    );
    let pack = soma_zero::OfficialCommitteeScenarioPackBuilder::default()
        .build(&crypto_pack_cfg)
        .expect("crypto pack");
    let pack_dir = common::output_dir("official-benchmark-crypto-pack-store");
    pack.write_to_dir(&pack_dir).expect("write pack");
    let outcome_path =
        common::output_dir("official-benchmark-crypto-outcomes").join("outcomes.json");
    official_committee_support::write_json(
        &outcome_path,
        serde_json::json!({
            "outcomes": [
                {
                    "symbol": "BTC-KRW",
                    "timestamp_ms": 1700000000000u64,
                    "horizon_bars": 24,
                    "triple_barrier_label": "TakeProfit",
                    "net_return_pct": 0.04,
                    "cost_bps": 5.0,
                    "slippage_bps": 2.0,
                    "source_kind": "OfficialApiCollected",
                    "no_lookahead_safe": true
                },
                {
                    "symbol": "ETH-KRW",
                    "timestamp_ms": 1700000000001u64,
                    "horizon_bars": 24,
                    "triple_barrier_label": "NoTradeCounterfactual",
                    "net_return_pct": -0.01,
                    "cost_bps": 5.0,
                    "slippage_bps": 2.0,
                    "source_kind": "OfficialApiCollected",
                    "no_lookahead_safe": true
                },
                {
                    "symbol": "XRP-KRW",
                    "timestamp_ms": 1700000000002u64,
                    "horizon_bars": 24,
                    "triple_barrier_label": "RiskDeniedCounterfactual",
                    "net_return_pct": -0.02,
                    "cost_bps": 5.0,
                    "slippage_bps": 2.0,
                    "source_kind": "OfficialApiCollected",
                    "no_lookahead_safe": true
                }
            ]
        }),
    );
    let baseline_path =
        common::output_dir("official-benchmark-crypto-baselines").join("baseline.json");
    official_committee_support::write_json(
        &baseline_path,
        serde_json::json!({
            "baseline_references": [
                {"symbol": "BTC-KRW", "timestamp_ms": 1700000000000u64, "horizon_bars": 24, "baseline_action": "Approve"},
                {"symbol": "ETH-KRW", "timestamp_ms": 1700000000001u64, "horizon_bars": 24, "baseline_action": "NoTrade"},
                {"symbol": "XRP-KRW", "timestamp_ms": 1700000000002u64, "horizon_bars": 24, "baseline_action": "ReduceSize"}
            ]
        }),
    );
    let linker_cfg_path = official_committee_support::write_linker_config(
        "official-benchmark-crypto",
        &CommitteeOutcomeLinkerConfig {
            linker_id: "official-benchmark-crypto-linker".to_string(),
            scenario_pack_path: Some(
                pack_dir
                    .join("official_scenario_pack.json")
                    .display()
                    .to_string(),
            ),
            outcome_artifact_paths: vec![outcome_path.display().to_string()],
            baseline_artifact_paths: vec![baseline_path.display().to_string()],
            output_root: common::output_dir("official-benchmark-crypto-linker-out")
                .display()
                .to_string(),
            strict_timestamp_match: true,
            max_timestamp_tolerance_ms: 0,
            require_same_symbol: true,
            require_same_horizon: true,
            reason_codes: vec![],
            ..CommitteeOutcomeLinkerConfig::default()
        },
    );
    let crypto = CommitteeOfficialBenchmarkRunner::default()
        .run(&official_committee_support::controlled_benchmark_config(
            "official-benchmark-crypto",
            &crypto_pack_cfg_path,
            &linker_cfg_path,
            false,
        ))
        .expect("crypto");
    assert_eq!(
        crypto.final_status,
        CommitteeOfficialBenchmarkFinalStatus::CryptoOnly
    );
}

#[test]
fn controlled_pack_runs_and_no_lookahead_violation_blocks_status() {
    let safe_pack_cfg =
        official_committee_support::controlled_pack_config("official-benchmark-safe", false);
    let safe_pack_cfg_path =
        official_committee_support::write_pack_config("official-benchmark-safe", &safe_pack_cfg);
    let safe_pack = soma_zero::OfficialCommitteeScenarioPackBuilder::default()
        .build(&safe_pack_cfg)
        .expect("safe pack");
    let safe_pack_dir = common::output_dir("official-benchmark-safe-pack-store");
    safe_pack
        .write_to_dir(&safe_pack_dir)
        .expect("write safe pack");
    let safe_linker_cfg_path = official_committee_support::write_linker_config(
        "official-benchmark-safe",
        &official_committee_support::controlled_linker_config(
            "official-benchmark-safe",
            &safe_pack_dir.join("official_scenario_pack.json"),
            true,
        ),
    );
    let cfg = official_committee_support::controlled_benchmark_config(
        "official-benchmark-safe",
        &safe_pack_cfg_path,
        &safe_linker_cfg_path,
        false,
    );
    let first = CommitteeOfficialBenchmarkRunner::default()
        .run(&cfg)
        .expect("first");
    let second = CommitteeOfficialBenchmarkRunner::default()
        .run(&cfg)
        .expect("second");
    assert_ne!(
        first.final_status,
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreOfficialRows
    );
    assert_eq!(first.to_text(), second.to_text());

    let blocked_pack_cfg =
        official_committee_support::controlled_pack_config("official-benchmark-blocked", false);
    let blocked_pack_cfg_path = official_committee_support::write_pack_config(
        "official-benchmark-blocked",
        &blocked_pack_cfg,
    );
    let blocked_pack = soma_zero::OfficialCommitteeScenarioPackBuilder::default()
        .build(&blocked_pack_cfg)
        .expect("blocked pack");
    let blocked_pack_dir = common::output_dir("official-benchmark-blocked-pack-store");
    blocked_pack
        .write_to_dir(&blocked_pack_dir)
        .expect("write blocked pack");
    let blocked_linker_cfg_path = official_committee_support::write_linker_config(
        "official-benchmark-blocked",
        &official_committee_support::controlled_linker_config(
            "official-benchmark-blocked",
            &blocked_pack_dir.join("official_scenario_pack.json"),
            false,
        ),
    );
    let blocked = CommitteeOfficialBenchmarkRunner::default()
        .run(&official_committee_support::controlled_benchmark_config(
            "official-benchmark-blocked",
            &blocked_pack_cfg_path,
            &blocked_linker_cfg_path,
            false,
        ))
        .expect("blocked");
    assert_eq!(
        blocked.final_status,
        CommitteeOfficialBenchmarkFinalStatus::CoreBlocked
    );
}
