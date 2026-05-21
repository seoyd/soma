mod support;

use soma_zero::{
    RealWorkspaceTimeoutAttributionConfig, Sprint93TimeoutAttributionRunner, TargetDirGrowthStatus,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_target_dir_growth.toml", name)
}

#[test]
fn target_dir_growth_is_sample_backed_and_local() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_target_dir_growth(&config("target-dir-growth"))
        .expect("report");
    assert_eq!(
        report.status,
        TargetDirGrowthStatus::TargetDirGrowthCaptured
    );
    assert_eq!(report.bytes_before, Some(1_048_576));
    assert_eq!(report.bytes_after, Some(1_054_720));
}
