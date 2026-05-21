#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    SafeConsolidationPatchV3Bundle, SafeConsolidationPatchV3Config, SafeConsolidationPatchV3Runner,
};

use super::shared_fixture_harness::{
    absolutize_fixture_path, example_fixture_path, load_json_fixture, named_output_dir,
};

pub fn sprint109_config_from_example(
    name: &str,
    output_name: &str,
) -> SafeConsolidationPatchV3Config {
    let mut config = SafeConsolidationPatchV3Config::from_toml_path(&example_fixture_path(name))
        .expect("parse sprint109 config");
    if let Some(paths) = config.sprint108_bundle_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize_fixture_path(path).display().to_string();
        }
    }
    config.output_root = named_output_dir("sprint109-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint109(name: &str, output_name: &str) -> SafeConsolidationPatchV3Bundle {
    let config = sprint109_config_from_example(name, output_name);
    SafeConsolidationPatchV3Runner::default()
        .run(&config)
        .expect("run sprint109")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint109_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint109-tests", name)
}
