#![allow(dead_code)]

use std::path::PathBuf;

use soma_zero::{
    EighteenArchetypePaperRotationConfig, Sprint102PaperRotationBundle,
    Sprint102PaperRotationRunner,
};

use super::sprint69_support::{absolutize, example_path, output_dir};

pub fn sprint102_config_from_example(
    name: &str,
    output_name: &str,
) -> EighteenArchetypePaperRotationConfig {
    let mut config = EighteenArchetypePaperRotationConfig::from_toml_path(&example_path(name))
        .expect("parse sprint102 config");
    for paths in [
        config.sprint101_bundle_paths.as_mut(),
        config.investor_archetype_card_paths.as_mut(),
        config.roster_plan_paths.as_mut(),
        config.regime_routing_paths.as_mut(),
        config.style_conflict_matrix_paths.as_mut(),
        config.paper_readiness_paths.as_mut(),
        config.workspace_acceptance_truth_paths.as_mut(),
    ] {
        if let Some(paths) = paths {
            for path in paths.iter_mut() {
                *path = absolutize(path).display().to_string();
            }
        }
    }
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint102(name: &str, output_name: &str) -> Sprint102PaperRotationBundle {
    let config = sprint102_config_from_example(name, output_name);
    Sprint102PaperRotationRunner::default()
        .run_sprint102_paper_rotation(&config)
        .expect("run sprint102")
}

pub fn sprint102_output_dir(name: &str) -> PathBuf {
    output_dir(name)
}
