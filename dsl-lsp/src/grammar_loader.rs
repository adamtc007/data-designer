use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::sync::{RwLock, mpsc};
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use futures_util::StreamExt;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRule {
    pub name: String,
    pub definition: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarMetadata {
    pub version: String,
    pub description: String,
    pub created: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    pub name: String,
    pub rules: Vec<GrammarRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub signature: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extensions {
    pub operators: HashMap<String, Vec<String>>,
    pub functions: Vec<FunctionDef>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarFile {
    pub metadata: GrammarMetadata,
    pub grammar: Grammar,
    pub extensions: Extensions,
}

#[derive(Debug, Clone)]
pub struct DynamicGrammar {
    pub keywords: Vec<String>,
    pub functions: Vec<(String, String)>,
    pub operators: Vec<(String, String)>,
    pub grammar_rules: Vec<GrammarRule>,
}

#[derive(Debug)]
pub struct GrammarLoader {
    grammar: Arc<RwLock<DynamicGrammar>>,
    grammar_file_path: PathBuf,
    last_modified: Arc<RwLock<Option<SystemTime>>>,
    reload_sender: Option<mpsc::UnboundedSender<()>>,
}

impl GrammarLoader {
    pub fn new(grammar_file_path: String) -> Self {
        let initial_grammar = DynamicGrammar {
            keywords: vec![],
            functions: vec![],
            operators: vec![],
            grammar_rules: vec![],
        };

        Self {
            grammar: Arc::new(RwLock::new(initial_grammar)),
            grammar_file_path: PathBuf::from(grammar_file_path),
            last_modified: Arc::new(RwLock::new(None)),
            reload_sender: None,
        }
    }

    pub async fn load_grammar(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(&self.grammar_file_path)?;
        let grammar_file: GrammarFile = serde_json::from_str(&content)?;

        let mut keywords = grammar_file.extensions.keywords.clone();

        // Extract function names from grammar rules
        let function_names: Vec<String> = grammar_file.grammar.rules
            .iter()
            .filter(|rule| rule.name.ends_with("_fn"))
            .map(|rule| {
                // Extract function name from rule definition
                // e.g., "SUBSTRING" from rule name "substring_fn"
                rule.name.trim_end_matches("_fn").to_uppercase()
            })
            .collect();

        keywords.extend(function_names);
        keywords.sort();
        keywords.dedup();

        // Build functions list with descriptions
        let functions: Vec<(String, String)> = grammar_file.extensions.functions
            .iter()
            .map(|f| (f.name.clone(), f.description.clone()))
            .collect();

        // Build operators list from extensions
        let mut operators = Vec::new();
        for (category, ops) in &grammar_file.extensions.operators {
            for op in ops {
                let description = match category.as_str() {
                    "arithmetic" => format!("{} arithmetic operation", op),
                    "string" => format!("{} string operation", op),
                    "comparison" => format!("{} comparison", op),
                    "logical" => format!("{} logical operation", op),
                    _ => format!("{} operator", op),
                };
                operators.push((op.clone(), description));
            }
        }

        let dynamic_grammar = DynamicGrammar {
            keywords,
            functions,
            operators,
            grammar_rules: grammar_file.grammar.rules,
        };

        let mut grammar = self.grammar.write().await;
        *grammar = dynamic_grammar;

        Ok(())
    }

    pub async fn get_keywords(&self) -> Vec<String> {
        let grammar = self.grammar.read().await;
        grammar.keywords.clone()
    }

    pub async fn get_functions(&self) -> Vec<(String, String)> {
        let grammar = self.grammar.read().await;
        grammar.functions.clone()
    }

    pub async fn get_operators(&self) -> Vec<(String, String)> {
        let grammar = self.grammar.read().await;
        grammar.operators.clone()
    }

    pub async fn get_grammar_rules(&self) -> Vec<GrammarRule> {
        let grammar = self.grammar.read().await;
        grammar.grammar_rules.clone()
    }

    pub async fn reload_if_changed(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // For now, just reload unconditionally
        // In a full implementation, you'd check file modification time
        self.load_grammar().await?;
        Ok(true)
    }

    pub fn get_grammar_path(&self) -> &Path {
        &self.grammar_file_path
    }
}

impl Default for GrammarLoader {
    fn default() -> Self {
        // Default to the grammar file in the project root
        let grammar_path = std::env::current_dir()
            .map(|mut path| {
                path.push("grammar_rules.json");
                path.to_string_lossy().to_string()
            })
            .unwrap_or_else(|_| "grammar_rules.json".to_string());

        Self::new(grammar_path)
    }
}