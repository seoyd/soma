#![allow(dead_code)]

use std::path::PathBuf;

use soma_zero::{
    InvestorArchetypeIngestionConfig, Sprint101InvestorArchetypeIngestionBundle,
    Sprint101InvestorArchetypeIngestionRunner,
};

use super::sprint69_support::{absolutize, example_path, output_dir};

pub fn sprint101_config_from_example(
    name: &str,
    output_name: &str,
) -> InvestorArchetypeIngestionConfig {
    let mut config = InvestorArchetypeIngestionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint101 config");
    if let Some(paths) = config.sprint100_bundle_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    if let Some(paths) = config.investor_material_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    if let Some(paths) = config.markdown_material_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    if let Some(paths) = config.candidate_table_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint101(name: &str, output_name: &str) -> Sprint101InvestorArchetypeIngestionBundle {
    let config = sprint101_config_from_example(name, output_name);
    Sprint101InvestorArchetypeIngestionRunner::default()
        .run_sprint101_investor_archetype_ingestion(&config)
        .expect("run sprint101")
}

pub fn sprint101_output_dir(name: &str) -> PathBuf {
    output_dir(name)
}
