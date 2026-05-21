mod common;

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use soma_zero::{
    KRXBoundedCollectionSmokeConfig, KRXOfficialCollectionClosureConfig,
    KRXOfficialCollectionClosureFinalStatus, KRXOfficialCollectionClosureRecommendation,
    KRXOfficialCollectionClosureRunner,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_krx_env(api_key: Option<&str>, endpoint: Option<&str>, run: impl FnOnce()) {
    let _guard = env_lock().lock().expect("env lock");
    let old_key = std::env::var_os("KRX_API_KEY");
    let old_endpoint = std::env::var_os("KRX_ENDPOINT_TEMPLATE");
    match api_key {
        Some(value) => unsafe { std::env::set_var("KRX_API_KEY", value) },
        None => unsafe { std::env::remove_var("KRX_API_KEY") },
    }
    match endpoint {
        Some(value) => unsafe { std::env::set_var("KRX_ENDPOINT_TEMPLATE", value) },
        None => unsafe { std::env::remove_var("KRX_ENDPOINT_TEMPLATE") },
    }
    run();
    match old_key {
        Some(value) => unsafe { std::env::set_var("KRX_API_KEY", value) },
        None => unsafe { std::env::remove_var("KRX_API_KEY") },
    }
    match old_endpoint {
        Some(value) => unsafe { std::env::set_var("KRX_ENDPOINT_TEMPLATE", value) },
        None => unsafe { std::env::remove_var("KRX_ENDPOINT_TEMPLATE") },
    }
}

fn config_from_example(name: &str, output_name: &str) -> KRXOfficialCollectionClosureConfig {
    let mut config = KRXOfficialCollectionClosureConfig::from_toml_path(&example_path(name))
        .expect("parse closure config");
    config.output_root = common::output_dir(output_name).display().to_string();
    config
}

#[test]
fn fixture_replay_stays_conservative() {
    let bundle = KRXOfficialCollectionClosureRunner::default()
        .run(&config_from_example(
            "soma_krx_collection_close_fixture_replay.toml",
            "krx-closure-fixture",
        ))
        .expect("run fixture replay closure");
    assert_eq!(
        bundle.collection_closure_report.final_status,
        KRXOfficialCollectionClosureFinalStatus::StillMissingOfficialCandles
    );
    assert!(bundle.raw_response_archive_summary.is_some());
}

#[test]
fn local_import_improves_outcome_links() {
    let bundle = KRXOfficialCollectionClosureRunner::default()
        .run(&config_from_example(
            "soma_krx_collection_close_local_import.toml",
            "krx-closure-local-import",
        ))
        .expect("run local import closure");
    assert_eq!(
        bundle.collection_closure_report.final_status,
        KRXOfficialCollectionClosureFinalStatus::KRXCompleteRowsImproved
    );
    assert_eq!(
        bundle.collection_closure_report.final_recommendation,
        KRXOfficialCollectionClosureRecommendation::CollectLongerKRXWindow
    );
    assert!(bundle.collection_closure_report.added_outcome_links > 0);
}

#[test]
fn missing_auth_blocks_collection_only_path() {
    with_krx_env(None, None, || {
        let mut config = config_from_example(
            "soma_krx_collection_close_live_disabled.toml",
            "krx-closure-missing-auth",
        );
        config.run_live_collection = true;
        let bundle = KRXOfficialCollectionClosureRunner::default()
            .run(&config)
            .expect("run missing auth closure");
        assert_eq!(
            bundle.collection_closure_report.final_status,
            KRXOfficialCollectionClosureFinalStatus::KRXAuthMissing
        );
    });
}

#[test]
fn budget_block_and_schema_block_are_reported() {
    let mut budget = config_from_example(
        "soma_krx_collection_close_local_import.toml",
        "krx-closure-budget-block",
    );
    budget.max_bytes = 16;
    let bundle = KRXOfficialCollectionClosureRunner::default()
        .run(&budget)
        .expect("run budget blocked closure");
    assert_eq!(
        bundle.collection_closure_report.final_status,
        KRXOfficialCollectionClosureFinalStatus::KRXBudgetBlocked
    );

    let output_dir = common::output_dir("krx-closure-schema-block-smoke");
    let bad_raw = output_dir.join("krx_005930_bad_raw.json");
    fs::write(&bad_raw, r#"{"symbol":"005930","timeframe":"1d","rows":[{"date":"bad","open":1.0,"high":2.0,"low":1.0,"close":1.5,"volume":1.0,"trade_value":1.0,"bid":1.0,"ask":1.0,"spread_bps":1.0}]}"#)
        .expect("write bad raw");
    let smoke = KRXBoundedCollectionSmokeConfig {
        smoke_id: "schema-block-smoke".to_string(),
        local_fixture_response_paths: vec![bad_raw.display().to_string()],
        output_root: output_dir.display().to_string(),
        require_krx_api_key: false,
        require_krx_endpoint_template: false,
        run_fixture_replay: true,
        run_local_import: false,
        ..KRXBoundedCollectionSmokeConfig::default()
    };
    let smoke_path = output_dir.join("schema_block_smoke.toml");
    fs::write(&smoke_path, smoke.to_toml_string().expect("smoke toml")).expect("write smoke toml");
    let config = KRXOfficialCollectionClosureConfig {
        run_id: "schema-block-run".to_string(),
        bounded_collection_smoke_config_path: Some(smoke_path.display().to_string()),
        output_root: output_dir.display().to_string(),
        run_fixture_replay: true,
        run_local_import: false,
        run_live_collection: false,
        ..KRXOfficialCollectionClosureConfig::default()
    };
    let bundle = KRXOfficialCollectionClosureRunner::default()
        .run(&config)
        .expect("run schema blocked closure");
    assert_eq!(
        bundle.collection_closure_report.final_status,
        KRXOfficialCollectionClosureFinalStatus::KRXSchemaBlocked
    );
}
