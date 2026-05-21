#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    SafeConsolidationPatchV4Bundle, SafeConsolidationPatchV4Config, SafeConsolidationPatchV4Runner,
};

use super::shared_fixture_harness::{
    absolutize_fixture_path, example_fixture_path, load_json_fixture, named_output_dir,
};

pub fn sprint110_config_from_example(
    name: &str,
    output_name: &str,
) -> SafeConsolidationPatchV4Config {
    let mut config = SafeConsolidationPatchV4Config::from_toml_path(&example_fixture_path(name))
        .expect("parse sprint110 config");
    if let Some(paths) = config.sprint109_bundle_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize_fixture_path(path).display().to_string();
        }
    }
    if let Some(paths) = config.sprint109_validation_summary_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize_fixture_path(path).display().to_string();
        }
    }
    config.output_root = named_output_dir("sprint110-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint110(name: &str, output_name: &str) -> SafeConsolidationPatchV4Bundle {
    let config = sprint110_config_from_example(name, output_name);
    SafeConsolidationPatchV4Runner::default()
        .run(&config)
        .expect("run sprint110")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint110_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint110-tests", name)
}
