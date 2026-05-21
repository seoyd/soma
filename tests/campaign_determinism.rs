mod common;

use std::fs;

use soma_zero::ResearchCampaignRunner;

#[test]
fn same_campaign_config_produces_identical_report() {
    let matrix = common::batch_matrix(
        "campaign-determinism-matrix",
        vec![common::dataset_entry(
            "valid",
            "generic_ohlcv_valid.csv",
            true,
        )],
        vec![common::baseline_variant("baseline_5m", true)],
    );
    let matrix_path = common::output_dir("campaign-determinism-matrix").join("matrix.toml");
    fs::write(&matrix_path, matrix.to_toml_string().expect("matrix toml")).expect("write matrix");
    let config = common::campaign_config(
        "campaign-determinism",
        vec![matrix_path.display().to_string()],
    );

    let first = ResearchCampaignRunner::default().run_campaign(&config);
    let second = ResearchCampaignRunner::default().run_campaign(&config);
    assert_eq!(first, second);
}
