#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    PaperRotationWarningClosureConfig, Sprint103PaperRotationClosureBundle,
    Sprint103PaperRotationClosureRunner,
};

use super::sprint69_support::{absolutize, example_path, output_dir, read_json};

pub fn sprint103_config_from_example(
    name: &str,
    output_name: &str,
) -> PaperRotationWarningClosureConfig {
    let mut config = PaperRotationWarningClosureConfig::from_toml_path(&example_path(name))
        .expect("parse sprint103 config");
    for paths in [
        config.sprint102_bundle_paths.as_mut(),
        config.paper_rotation_paths.as_mut(),
        config.lower_confidence_paths.as_mut(),
        config.weak_source_review_paths.as_mut(),
        config.proposal_run_paths.as_mut(),
        config.entry_timing_paths.as_mut(),
        config.debate_session_paths.as_mut(),
        config.chairman_synthesis_paths.as_mut(),
        config.risk_governor_handoff_paths.as_mut(),
        config.paper_trace_paths.as_mut(),
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

pub fn run_sprint103(name: &str, output_name: &str) -> Sprint103PaperRotationClosureBundle {
    let config = sprint103_config_from_example(name, output_name);
    Sprint103PaperRotationClosureRunner::default()
        .run(&config)
        .expect("run sprint103")
}

pub fn sprint103_output_dir(name: &str) -> PathBuf {
    output_dir(name)
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    read_json(example_path(name))
}
