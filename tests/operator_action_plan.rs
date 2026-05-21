use soma_zero::{
    OfficialEvidenceExpansionReport, OperatorActionPriority, ProviderAuthEnvRequirement,
    ProviderAuthPreflightConfig, ProviderAuthPreflightRunner, ProviderKind, ReasonCode,
    build_operator_action_plan,
};

fn alpha_var(prefix: &str) -> String {
    format!("SOMA_TEST_ACTION_{prefix}_ALPHA")
}

fn krx_key_var(prefix: &str) -> String {
    format!("SOMA_TEST_ACTION_{prefix}_KRX_KEY")
}

fn krx_endpoint_var(prefix: &str) -> String {
    format!("SOMA_TEST_ACTION_{prefix}_KRX_ENDPOINT")
}

fn auth_report(
    prefix: &str,
    alpha_present: bool,
    krx_key_present: bool,
    krx_endpoint_present: bool,
) -> soma_zero::ProviderAuthPreflightReport {
    unsafe {
        std::env::remove_var(alpha_var(prefix));
        std::env::remove_var(krx_key_var(prefix));
        std::env::remove_var(krx_endpoint_var(prefix));
        if alpha_present {
            std::env::set_var(alpha_var(prefix), "present");
        }
        if krx_key_present {
            std::env::set_var(krx_key_var(prefix), "present");
        }
        if krx_endpoint_present {
            std::env::set_var(krx_endpoint_var(prefix), "template");
        }
    }
    ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::KrxOpenApi, ProviderKind::AlphaVantage],
        required_env_vars: vec![
            ProviderAuthEnvRequirement {
                provider_kind: ProviderKind::KrxOpenApi,
                api_key_env_var: Some(krx_key_var(prefix)),
                api_secret_env_var: None,
                endpoint_template_env_var: Some(krx_endpoint_var(prefix)),
            },
            ProviderAuthEnvRequirement {
                provider_kind: ProviderKind::AlphaVantage,
                api_key_env_var: Some(alpha_var(prefix)),
                api_secret_env_var: None,
                endpoint_template_env_var: None,
            },
        ],
        ..ProviderAuthPreflightConfig::default()
    })
}

#[test]
fn missing_auth_creates_expected_actions() {
    let alpha_missing = build_operator_action_plan(
        &auth_report("ALPHA", false, true, true),
        None,
        false,
        true,
        false,
    );
    let krx_key_missing = build_operator_action_plan(
        &auth_report("KRXKEY", true, false, true),
        None,
        false,
        true,
        false,
    );
    let krx_endpoint_missing = build_operator_action_plan(
        &auth_report("KRXENDPOINT", true, true, false),
        None,
        false,
        true,
        false,
    );

    assert!(
        alpha_missing
            .missing_auth_actions
            .contains(&"set-alphavantage-auth".to_string())
    );
    assert!(
        krx_key_missing
            .missing_auth_actions
            .contains(&"set-krx-auth".to_string())
    );
    assert!(
        krx_endpoint_missing
            .missing_auth_actions
            .contains(&"set-krx-endpoint-template".to_string())
    );
}

#[test]
fn action_plan_contains_no_secret_values() {
    unsafe { std::env::set_var(alpha_var("SECRET"), "super-secret-value") };
    let plan = build_operator_action_plan(
        &auth_report("SECRET", false, true, true),
        None,
        false,
        true,
        false,
    );
    let text = plan.to_text();

    assert!(!text.contains("super-secret-value"));
    assert!(text.contains("ALPHAVANTAGE_API_KEY"));
    unsafe { std::env::remove_var(alpha_var("SECRET")) };
}

#[test]
fn suggested_commands_are_local_and_research_only() {
    let plan = build_operator_action_plan(
        &auth_report("COMMANDS", false, true, false),
        None,
        true,
        true,
        true,
    );

    assert!(plan.actions.iter().all(|action| {
        action
            .command_suggestion
            .as_deref()
            .is_none_or(|command| !command.contains("://"))
    }));
    assert!(plan.actions.iter().all(|action| {
        action
            .command_suggestion
            .as_deref()
            .is_none_or(|command| !command.contains("broker") && !command.contains("account"))
    }));
}

#[test]
fn action_plan_is_deterministic() {
    let first = serde_json::to_string(&build_operator_action_plan(
        &auth_report("DETERMINISM", false, true, false),
        None::<&OfficialEvidenceExpansionReport>,
        true,
        true,
        false,
    ))
    .expect("first");
    let second = serde_json::to_string(&build_operator_action_plan(
        &auth_report("DETERMINISM", false, true, false),
        None::<&OfficialEvidenceExpansionReport>,
        true,
        true,
        false,
    ))
    .expect("second");

    assert_eq!(first, second);
}

#[test]
fn crypto_run_action_is_recommended() {
    let plan = build_operator_action_plan(
        &auth_report("CRYPTO", false, true, false),
        None,
        false,
        true,
        false,
    );
    let action = plan
        .actions
        .iter()
        .find(|action| action.action_id == "run-crypto-only-evidence")
        .expect("crypto action");

    assert_eq!(action.priority, OperatorActionPriority::Recommended);
    assert!(
        action
            .reason_codes
            .contains(&ReasonCode::OperatorActionPlanBuilt)
    );
}
