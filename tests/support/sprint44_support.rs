#![allow(dead_code)]

use std::path::Path;

use soma_zero::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceRow, OfficialCandleCoveragePack,
    OfficialCandleCoveragePackConfig, OfficialCandleJoinAuditConfig,
    OfficialReadyMatchClosureConfig,
};

pub fn load_bundle(path: &str) -> ComparableCommitteeEvidenceBundle {
    ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path)).expect("bundle")
}

pub fn load_row(path: &str) -> ComparableCommitteeEvidenceRow {
    load_bundle(path).rows.into_iter().next().expect("row")
}

pub fn load_pack(path: &str) -> OfficialCandleCoveragePack {
    let config =
        OfficialCandleCoveragePackConfig::from_toml_path(Path::new(path)).expect("pack cfg");
    OfficialCandleCoveragePack::build(&config).expect("pack")
}

pub fn load_audit_config(path: &str) -> OfficialCandleJoinAuditConfig {
    OfficialCandleJoinAuditConfig::from_toml_path(Path::new(path)).expect("audit cfg")
}

pub fn load_closure_config(path: &str) -> OfficialReadyMatchClosureConfig {
    OfficialReadyMatchClosureConfig::from_toml_path(Path::new(path)).expect("closure cfg")
}
