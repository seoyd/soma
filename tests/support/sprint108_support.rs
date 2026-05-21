#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    SafeConsolidationPatchV2Bundle, SafeConsolidationPatchV2Config, SafeConsolidationPatchV2Runner,
};

use super::shared_fixture_harness::{
    absolutize_fixture_path, example_fixture_path, load_json_fixture, named_output_dir,
};

pub fn sprint108_config_from_example(
    name: &str,
    output_name: &str,
) -> SafeConsolidationPatchV2Config {
    let mut config = SafeConsolidationPatchV2Config::from_toml_path(&example_fixture_path(name))
        .expect("parse sprint108 config");
    if let Some(paths) = config.sprint107_bundle_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize_fixture_path(path).display().to_string();
        }
    }
    config.output_root = named_output_dir("sprint108-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint108(name: &str, output_name: &str) -> SafeConsolidationPatchV2Bundle {
    let config = sprint108_config_from_example(name, output_name);
    SafeConsolidationPatchV2Runner::default()
        .run(&config)
        .expect("run sprint108")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint108_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint108-tests", name)
}
