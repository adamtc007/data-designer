use data_designer_core::{manager, engine::{RulesEngine, Facts}};
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ... (Other use statements and struct definitions from your repo)
#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Executes a test case for the derived attribute rules.
    TestRule {
        /// The path to the test case directory.
        #[arg(long)]
        case: PathBuf,
    },
    // ... other commands
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::TestRule { case } => {
            println!("🧪 Running test case '{}'", case.display());

            let dict = manager::load_dictionary()?;
            let engine = RulesEngine::new(&dict)?;

            let source_path = case.join("source_facts.json");
            let source_content = fs::read_to_string(&source_path)?;
            let initial_facts: Facts = serde_json::from_str(&source_content)?;
            println!("  -> Loaded {} source fact(s).", initial_facts.len());

            let target_path = case.join("expected_targets.json");
            let target_content = fs::read_to_string(&target_path)?;
            let expected_targets: HashMap<String, Value> = serde_json::from_str(&target_content)?;
            let targets_to_calculate: Vec<String> = expected_targets.keys().cloned().collect();

            let final_facts = engine.evaluate_chain(&targets_to_calculate, &initial_facts)?;
            
            let mut all_tests_passed = true;
            for (target_name, expected_value) in expected_targets {
                if let Some(actual_value) = final_facts.get(&target_name) {
                    // We need to convert our native Value enum to serde_json::Value for comparison
                    let actual_serde_value = match actual_value {
                        data_designer_core::models::Value::Integer(i) => Value::from(*i),
                        data_designer_core::models::Value::String(s) => Value::from(s.clone()),
                        data_designer_core::models::Value::Boolean(b) => Value::from(*b),
                        _ => Value::Null,
                    };

                    if actual_serde_value == expected_value {
                        println!("  ✅ PASS: '{}' matches expected value '{}'.", target_name, expected_value);
                    } else {
                        println!("  🔥 FAIL: '{}' was '{}', but expected '{}'.", target_name, actual_serde_value, expected_value);
                        all_tests_passed = false;
                    }
                } else {
                    println!("  🔥 FAIL: Target '{}' was not calculated.", target_name);
                    all_tests_passed = false;
                }
            }
            
            if !all_tests_passed {
                bail!("One or more tests failed.");
            }
        }
        // ... other command handlers
    }
    Ok(())
}
