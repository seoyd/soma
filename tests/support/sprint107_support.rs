#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    SafeConsolidationPatchV1Bundle, SafeConsolidationPatchV1Config, SafeConsolidationPatchV1Runner,
};

use super::shared_fixture_harness::{
    absolutize_fixture_path, example_fixture_path, load_json_fixture, named_output_dir,
};

pub fn sprint107_config_from_example(
    name: &str,
    output_name: &str,
) -> SafeConsolidationPatchV1Config {
    let mut config = SafeConsolidationPatchV1Config::from_toml_path(&example_fixture_path(name))
        .expect("parse sprint107 config");
    if let Some(paths) = config.sprint106_bundle_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize_fixture_path(path).display().to_string();
        }
    }
    config.output_root = named_output_dir("sprint107-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint107(name: &str, output_name: &str) -> SafeConsolidationPatchV1Bundle {
    let config = sprint107_config_from_example(name, output_name);
    SafeConsolidationPatchV1Runner::default()
        .run(&config)
        .expect("run sprint107")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint107_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint107-tests", name)
}
