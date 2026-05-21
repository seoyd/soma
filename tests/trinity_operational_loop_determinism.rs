use std::path::Path;

use soma_zero::{TrinityCommitteeOperationalLoopConfig, TrinityOperationalLoopRunner};

#[test]
fn trinity_operational_loop_is_deterministic() {
    let config = TrinityCommitteeOperationalLoopConfig::from_toml_path(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/soma_trinity_operational_loop_kis.toml"
    )))
    .unwrap();
    let first = TrinityOperationalLoopRunner::default()
        .run(&config)
        .unwrap();
    let second = TrinityOperationalLoopRunner::default()
        .run(&config)
        .unwrap();
    assert_eq!(first.report.fingerprint, second.report.fingerprint);
    assert_eq!(
        first.operational_audit_timeline.fingerprint,
        second.operational_audit_timeline.fingerprint
    );
    assert_eq!(
        first.candidate_generation_report.fingerprint,
        second.candidate_generation_report.fingerprint
    );
}
