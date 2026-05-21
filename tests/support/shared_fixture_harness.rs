#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn load_json_fixture<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read json fixture"))
        .expect("parse json fixture")
}

pub fn example_fixture_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn absolutize_fixture_path(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        project_root().join(candidate)
    }
}

pub fn load_toml_fixture<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    toml::from_str(&fs::read_to_string(path).expect("read toml fixture"))
        .expect("parse toml fixture")
}

pub fn load_csv_fixture(path: impl AsRef<Path>) -> Vec<BTreeMap<String, String>> {
    let text = fs::read_to_string(path).expect("read csv fixture");
    let mut lines = text.lines();
    let headers = lines
        .next()
        .expect("csv header row")
        .split(',')
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            headers
                .iter()
                .cloned()
                .zip(line.split(',').map(|value| value.trim().to_string()))
                .collect::<BTreeMap<_, _>>()
        })
        .collect()
}

pub fn temp_output_dir_for_test(test_name: &str) -> PathBuf {
    named_output_dir("shared-fixture-harness-tests", test_name)
}

pub fn named_output_dir(namespace: &str, test_name: &str) -> PathBuf {
    let sanitized = test_name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    let path = project_root()
        .join("target")
        .join(namespace)
        .join(sanitized);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create shared fixture harness output");
    path
}

pub fn write_json_output<T: Serialize>(
    output_dir: impl AsRef<Path>,
    file_name: &str,
    value: &T,
) -> String {
    let path = output_dir.as_ref().join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize shared json output");
    fs::write(&path, text).expect("write shared json output");
    path.display().to_string()
}

pub fn write_text_output(output_dir: impl AsRef<Path>, file_name: &str, value: &str) -> String {
    let path = output_dir.as_ref().join(file_name);
    fs::write(&path, value).expect("write shared text output");
    path.display().to_string()
}

pub fn assert_deterministic_text(first: &str, second: &str) {
    assert_eq!(first, second, "text output must be deterministic");
}

pub fn assert_no_secret_like_values(text: &str) {
    let lowercase = text.to_ascii_lowercase();
    for forbidden in ["api_key", "api-secret", "secret=", "token=", "bearer "] {
        assert!(
            !lowercase.contains(forbidden),
            "secret-like value leaked: {forbidden}"
        );
    }
}

pub fn assert_no_order_account_fields(text: &str) {
    let lowercase = text.to_ascii_lowercase();
    for forbidden in [
        "order_id",
        "orderable",
        "account_id",
        "account_balance",
        "buying_power",
        "positions",
        "holdings",
    ] {
        assert!(
            !lowercase.contains(forbidden),
            "order/account field leaked: {forbidden}"
        );
    }
}

pub fn assert_no_runtime_fields(text: &str) {
    let lowercase = text.to_ascii_lowercase();
    for forbidden in [
        "runtime_enabled",
        "training_enabled",
        "live_inference",
        "live_trading",
        "train-model",
        "mamba runtime",
        "gated deltanet runtime",
    ] {
        assert!(
            !lowercase.contains(forbidden),
            "runtime/training field leaked: {forbidden}"
        );
    }
}

pub fn assert_source_boundary_preserved(value: &Value) {
    match value {
        Value::Object(map) => {
            if let Some(flag) = map
                .get("source_boundary_fields_present")
                .and_then(Value::as_bool)
            {
                assert!(flag, "source boundary fields must remain preserved");
            }
            for nested in map.values() {
                assert_source_boundary_preserved(nested);
            }
        }
        Value::Array(values) => {
            for nested in values {
                assert_source_boundary_preserved(nested);
            }
        }
        _ => {}
    }
}

pub fn assert_no_lookahead_preserved(value: &Value) {
    match value {
        Value::Object(map) => {
            if let Some(flag) = map
                .get("no_lookahead_fields_present")
                .and_then(Value::as_bool)
            {
                assert!(flag, "no-lookahead fields must remain preserved");
            }
            for nested in map.values() {
                assert_no_lookahead_preserved(nested);
            }
        }
        Value::Array(values) => {
            for nested in values {
                assert_no_lookahead_preserved(nested);
            }
        }
        _ => {}
    }
}
