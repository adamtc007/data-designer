use onboarding::{compile_onboard, CompileInputs, ExecutionConfig, execute_plan};
use onboarding::ast::oodl::OnboardIntent;
use onboarding::meta::loader::load_from_dir;
use std::path::Path;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let meta = load_from_dir(Path::new("onboarding/metadata"))?;
    let intent = OnboardIntent {
        instance_id: "OR-2025-00042".into(),
        cbu_id: "CBU-12345".into(),
        products: vec!["GlobalCustody@v3".into()],
    };

    let inputs = CompileInputs {
        intent: &intent,
        meta: &meta,
        team_users: vec![
            serde_json::json!({"email":"ops.admin@client.com","role":"Administrator"}),
            serde_json::json!({"email":"ops.approver@client.com","role":"Approver"}),
        ],
        cbu_profile: serde_json::json!({"region":"EU"}),
    };

    let out = compile_onboard(inputs)?;
    println!("--- PLAN ---\n{}", serde_json::to_string_pretty(&out.plan)?);
    println!("--- IDD ---\n{}", serde_json::to_string_pretty(&out.idd)?);

    let cfg = ExecutionConfig{};
    execute_plan(&out.plan, &cfg).await?;
    Ok(())
}
